use anyhow::{Context, Result};
use goblin::elf::{Elf, SectionHeader, Sym};
use log::debug;
use memmap2::Mmap;
use postcard_schema::schema::owned::{
    OwnedData, OwnedDataModelType, OwnedNamedField, OwnedVariant,
};
use std::{fs::File, path::Path};

// Tries to recover a postcard schema from an ELF file
// `RUSTFLAGS=-Zprint-type-sizes  cargo +nightly build > out.txt` helped a lot to understand the layout of the types

pub struct SchemaRestorer {
    // Use a reference to the mmap data for the Elf
    elf: Elf<'static>,
    mmap: Mmap,
}

impl SchemaRestorer {
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self> {
        // Memory map the file for efficient access
        let file = File::open(path)?;
        let mmap = unsafe { Mmap::map(&file)? };

        // Convert mmap to static lifetime - safe because we keep mmap alive with the struct
        let mmap_slice: &'static [u8] = unsafe { std::mem::transmute(mmap.as_ref()) };
        let elf = Elf::parse(mmap_slice).context("Failed to parse ELF file")?;

        Ok(Self { elf, mmap })
    }

    fn find_symbol(&self, symbol: &str) -> Result<Sym> {
        self.elf
            .syms
            .iter()
            .find(|sym| {
                self.elf
                    .strtab
                    .get_at(sym.st_name)
                    .map(|name| name == symbol)
                    .unwrap_or(false)
            })
            .ok_or_else(|| anyhow::anyhow!("Could not find {} symbol", symbol))
    }

    fn find_section_by_index(&self, index: usize) -> Result<&SectionHeader> {
        self.elf
            .section_headers
            .get(index)
            .ok_or_else(|| anyhow::anyhow!("Section index {} out of bounds", index))
    }

    // Convert a section-relative address to file offset
    fn section_addr_to_offset(&self, section_idx: usize, addr: u64) -> Result<usize> {
        let section = self.find_section_by_index(section_idx)?;
        let offset = section.sh_offset + addr - section.sh_addr;
        Ok(offset as usize)
    }

    fn read_u32_at(&self, offset: usize) -> Result<usize> {
        let bytes = self
            .mmap
            .get(offset..offset + 4)
            .context("Failed to read u32: out of bounds")?;
        Ok(u32::from_le_bytes(bytes.try_into()?) as usize)
    }

    // This function resolves a pointer in the ELF file, considering section boundaries
    fn read_pointer_at(&self, offset: usize) -> Result<usize> {
        let addr = self.read_u32_at(offset)? as u64;
        debug!(
            "Resolving pointer at offset {:#x} to addr {:#x}",
            offset, addr
        );

        // Find the section containing this address
        for (idx, section) in self.elf.section_headers.iter().enumerate() {
            let section_start = section.sh_addr;
            let section_end = section_start + section.sh_size;
            let section_name = self.elf.shdr_strtab.get_at(section.sh_name).unwrap_or("");
            if addr >= section_start && addr < section_end {
                debug!(
                    "Resolving offset {:#x} to pointer at address {:#x} section {} ({})",
                    offset, addr, section_name, idx
                );
                // Convert virtual address to file offset
                return self.section_addr_to_offset(idx, addr);
            }
        }

        Err(anyhow::anyhow!(
            "Could not resolve pointer address {:#x}",
            addr
        ))
    }

    fn decode_data_model_type(&self, offset: usize) -> Result<OwnedDataModelType> {
        let type_tag = self.mmap[offset];
        debug!("Decoding type tag {} at offset {:#x}", type_tag, offset);

        // The discriminant of the enum DataModelType is a bit special:
        // It starts at 4, then follows the variant order as defined in the enum
        // The struct variant contains another enum with 4 variants, so the struct variant is left away, and mapped to discriminator 0..4

        // Here we follow the order of the enum DataModelType as defined in the source code
        match type_tag {
            // 1. Bool
            4 => Ok(OwnedDataModelType::Bool),

            // 2. I8
            5 => Ok(OwnedDataModelType::I8),

            // 3. U8
            6 => Ok(OwnedDataModelType::U8),

            // 4. I16
            7 => Ok(OwnedDataModelType::I16),

            // 5. I32
            8 => Ok(OwnedDataModelType::I32),

            // 6. I64
            9 => Ok(OwnedDataModelType::I64),

            // 7. I128
            10 => Ok(OwnedDataModelType::I128),

            // 8. U16
            11 => Ok(OwnedDataModelType::U16),

            // 9. U32
            12 => Ok(OwnedDataModelType::U32),

            // 10. U64
            13 => Ok(OwnedDataModelType::U64),

            // 11. U128
            14 => Ok(OwnedDataModelType::U128),

            // 12. Usize
            15 => Ok(OwnedDataModelType::Usize),

            // 13. Isize
            16 => Ok(OwnedDataModelType::Isize),

            // 14. F32
            17 => Ok(OwnedDataModelType::F32),

            // 15. F64
            18 => Ok(OwnedDataModelType::F64),

            // 16. Char
            19 => Ok(OwnedDataModelType::Char),

            // 17. String
            20 => Ok(OwnedDataModelType::String),

            // 18. ByteArray
            21 => Ok(OwnedDataModelType::ByteArray),

            // 19. Option
            22 => {
                let inner = self.decode_data_model_type(self.read_pointer_at(offset + 4)?)?;
                Ok(OwnedDataModelType::Option(Box::new(inner)))
            }

            // 20. Unit
            23 => Ok(OwnedDataModelType::Unit),

            // 21. Seq
            24 => {
                let inner = self.decode_data_model_type(self.read_pointer_at(offset + 4)?)?;
                Ok(OwnedDataModelType::Seq(Box::new(inner)))
            }

            // 22. Tuple
            25 => {
                let types = self.decode_slice(offset + 4, Self::decode_data_model_type)?;
                Ok(OwnedDataModelType::Tuple(types))
            }

            // 23. Map
            26 => {
                /*
                print-type-size     variant `Map`: 12 bytes
                print-type-size         padding: 4 bytes
                print-type-size         field `.key`: 4 bytes, alignment: 4 bytes
                print-type-size         field `.val`: 4 bytes
                */

                let key = self.decode_data_model_type(self.read_pointer_at(offset + 4)?)?;
                let value = self.decode_data_model_type(self.read_pointer_at(offset + 8)?)?;

                Ok(OwnedDataModelType::Map {
                    key: Box::new(key),
                    val: Box::new(value),
                })
            }

            // 24. Struct
            0..4 => {
                /*
                print-type-size type: `destore::DataModelType`: 20 bytes, alignment: 4 bytes
                print-type-size     variant `Struct`: 20 bytes
                print-type-size         field `.data`: 12 bytes
                print-type-size         field `.name`: 8 bytes
                */
                let data = self.decode_data(offset)?;
                let name = self.decode_static_str(offset + 12)?;
                Ok(OwnedDataModelType::Struct { name, data })
            }

            // 27 =>  Struct is left away, because it is in 0..4

            // 25. Enum
            28 => {
                /*
                   print-type-size     variant `Enum`: 20 bytes
                   print-type-size         padding: 4 bytes
                   print-type-size         field `.name`: 8 bytes, alignment: 4 bytes
                   print-type-size         field `.variants`: 8 bytes
                */

                let name = self.decode_static_str(offset + 4)?; // takes two words
                let variants = self.decode_slice(offset + 12, Self::decode_variant)?;
                Ok(OwnedDataModelType::Enum { name, variants })
            }

            // 29. Schema
            29 => Ok(OwnedDataModelType::Schema),

            _ => anyhow::bail!("Unknown type tag: {}", type_tag),
        }
    }

    fn decode_static_str(&self, offset: usize) -> Result<Box<str>> {
        let name_str_ptr = self.read_pointer_at(offset)?;
        let name_str_len = self.read_u32_at(offset + 4)?;

        let name_bytes = &self.mmap[name_str_ptr..(name_str_ptr + name_str_len)];
        Ok(std::str::from_utf8(name_bytes)?
            .to_string()
            .into_boxed_str())
    }

    fn decode_variant(&self, offset: usize) -> Result<OwnedVariant> {
        /*
        print-type-size type: `postcard_schema::schema::Variant`: 20 bytes, alignment: 4 bytes
        print-type-size     field `.data`: 12 bytes
        print-type-size     field `.name`: 8 bytes
         */

        let name = self.decode_static_str(offset + 12)?;
        let data = self.decode_data(offset)?;

        Ok(OwnedVariant { name, data })
    }

    fn decode_data(&self, offset: usize) -> Result<OwnedData> {
        /*
        print-type-size type: `postcard_schema::schema::Data`: 12 bytes, alignment: 4 bytes
        print-type-size     discriminant: 4 bytes
        print-type-size     variant `Tuple`: 8 bytes
        print-type-size         field `.0`: 8 bytes
        print-type-size     variant `Struct`: 8 bytes
        print-type-size         field `.0`: 8 bytes
        print-type-size     variant `Newtype`: 4 bytes
        print-type-size         field `.0`: 4 bytes
        print-type-size     variant `Unit`: 0 bytes
         */

        let discriminant = self.read_u32_at(offset)?;
        match discriminant {
            0 => Ok(OwnedData::Unit),
            1 => {
                // Newtype
                let inner = self.decode_data_model_type(self.read_pointer_at(offset + 4)?)?;
                Ok(OwnedData::Newtype(Box::new(inner)))
            }
            2 => {
                // Tuple
                let inner = self.decode_slice(offset + 4, Self::decode_data_model_type)?;
                Ok(OwnedData::Tuple(inner))
            }
            3 => {
                // Struct
                let inner = self.decode_slice(offset + 4, Self::decode_named_field)?;
                Ok(OwnedData::Struct(inner))
            }
            _ => anyhow::bail!("Unknown data discriminant: {}", discriminant),
        }
    }

    fn decode_named_field(&self, offset: usize) -> Result<OwnedNamedField> {
        /*
        print-type-size type: `postcard_schema::schema::NamedField`: 12 bytes, alignment: 4 bytes
        print-type-size     field `.name`: 8 bytes
        print-type-size     field `.ty`: 4 bytes
         */
        let name = self.decode_static_str(offset)?;
        let ty = self.decode_data_model_type(self.read_pointer_at(offset + 8)?)?;
        Ok(OwnedNamedField { name, ty })
    }

    fn decode_slice<F, R>(&self, offset: usize, inner_decoder: F) -> Result<Box<[R]>>
    where
        F: Fn(&Self, usize) -> Result<R>,
    {
        let slice_start = self.read_pointer_at(offset)?;
        let count = self.read_u32_at(offset + 4)?;
        debug!("slice start {:#x} count {}", slice_start, count);
        let mut types_vec = Vec::with_capacity(count);
        for i in 0..count {
            let type_offset = self.read_pointer_at(slice_start + i * 4)?;
            let inner_type = inner_decoder(self, type_offset)?;
            types_vec.push(inner_type);
        }
        Ok(types_vec.into_boxed_slice())
    }

    pub fn load_schema_from_symbol(&self, symbol: &str) -> Result<OwnedDataModelType> {
        let schema_sym = self.find_symbol(symbol)?;
        let section_idx = schema_sym.st_shndx;

        // Convert the symbol's value to a file offset
        let schema_offset = self.section_addr_to_offset(section_idx, schema_sym.st_value)?;
        debug!(
            "sym {} value: {:#x}, offset: {:#x}",
            symbol, schema_sym.st_value, schema_offset
        );
        let type_def = self.decode_data_model_type(self.read_pointer_at(schema_offset)?)?;

        Ok(type_def)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use log::info;
    use std::path::Path;
    use std::process::Command;

    #[test]
    fn test_restore() {
        env_logger::try_init_from_env(env_logger::Env::default().default_filter_or("debug")).ok();
        let elf_path = build_riscv32_elf();
        let restorer = SchemaRestorer::from_path(&elf_path).unwrap();

        let tests = &[
            ("_DESTORE_SCHEMA_BOOL", "bool"),
            ("_DESTORE_SCHEMA_I8", "i8"),
            ("_DESTORE_SCHEMA_U8", "u8"),
            ("_DESTORE_SCHEMA_I16", "i16"),
            ("_DESTORE_SCHEMA_I32", "i32"),
            ("_DESTORE_SCHEMA_I64", "i64"),
            ("_DESTORE_SCHEMA_I128", "i128"),
            ("_DESTORE_SCHEMA_U16", "u16"),
            ("_DESTORE_SCHEMA_U32", "u32"),
            ("_DESTORE_SCHEMA_U64", "u64"),
            ("_DESTORE_SCHEMA_U128", "u128"),
            // ("_DESTORE_SCHEMA_USIZE", "usize"),
            //("_DESTORE_SCHEMA_ISIZE", "isize"),
            ("_DESTORE_SCHEMA_F32", "f32"),
            ("_DESTORE_SCHEMA_F64", "f64"),
            ("_DESTORE_SCHEMA_CHAR", "char"),
            ("_DESTORE_SCHEMA_STRING", "String"),
            //("_DESTORE_SCHEMA_BYTEARRAY", "&[u8]"),
            ("_DESTORE_SCHEMA_OPTION", "Option<u8>"),
            ("_DESTORE_SCHEMA_UNIT", "()"),
            ("_DESTORE_SCHEMA_SEQUENCE", "[u16]"),
            ("_DESTORE_SCHEMA_TUPLE", "(u8, u16, u32)"),
            ("_DESTORE_SCHEMA_MAP", "Map<u32, String>"),
            ("_DESTORE_SCHEMA_UNITSTRUCT", "struct UnitStruct"),
            (
                "_DESTORE_SCHEMA_NEWTYPESTRUCT",
                "struct NewTypeStruct(String)",
            ),
            (
                "_DESTORE_SCHEMA_TUPLESTRUCT",
                "struct TupStruct(u64, String)",
            ),
            (
                "_DESTORE_SCHEMA_STRUCTSTRUCT",
                "struct Classic { a: u32, b: u16, c: bool }",
            ),
            (
                "_DESTORE_SCHEMA_ENUM",
                "enum Enums { Unit, Nt(u64), Tup(u32, bool), Str { a: u32, b: u16, c: bool } }",
            ),
            ("_DESTORE_SCHEMA_SCHEMA", "Schema"),
        ];
        for (symbol, expected) in tests {
            info!("testing symbol {}", symbol);
            let schema = restorer.load_schema_from_symbol(symbol).unwrap();
            let actual = schema.to_pseudocode();
            if actual != *expected {
                panic!(
                    "Testing symbol {} failed. ex: {} ac: {}",
                    symbol, expected, actual
                );
            }

            //assert_eq!(actual, expected);
        }
    }

    fn build_riscv32_elf() -> &'static Path {
        let output = Command::new("cargo")
            .args([
                "build",
                "--target",
                "riscv32imac-unknown-none-elf",
                "--package",
                "elf-schema-restore-test",
            ])
            .output()
            .expect("Failed to build riscv32 binary");

        assert!(
            output.status.success(),
            "Failed to build riscv32 binary: {:?}",
            String::from_utf8_lossy(&output.stderr)
        );

        let elf_path =
            Path::new("../target/riscv32imac-unknown-none-elf/debug/elf-schema-restore-test");
        assert!(
            elf_path.exists(),
            "ELF file was not generated at expected path: {:?}",
            elf_path
        );

        println!("Successfully built ELF: {:?}", elf_path);
        elf_path
    }
}
