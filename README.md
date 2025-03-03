# Destore

Store postcard- and/or defmt-encoded data in the flash of your MCU and retrieve it later from the host with a debugger.
The host recovers the postard-schema and defmt tables from the ELF file.

### WARNING: Experimental / Proof of Concept

**Note**: You may want to use poststation directly, if you want to transfer data from the MCU to the host via
USB/serial.

## Organization

* [destore](./destore): lib crate that can used on the MCU to store records in the flash memory.
* [example](./example): Example Application
* [destore-tools](./destore-tools): CLI + lib to use on the host to:
    * recover records from the flash memory
    * extract metadata (such as the employed postcard-schema) from the ELF file

## Usage

1. Cargo install `destore-tools`. This installs the `destore` cli
2. In your firmware project:
    * Create an enum that represents the records you want to store.
    * `destore::export_schema!` to export the record type in the elf. (Makes the postcard-schema available to the host)
    * Use `destore::Storer` to store records in a predefined flash region.
3. Add `destore proxy -- ` to the front of your cargo runner:  
   e.g. `runner = "destore proxy -- espflash flash --monitor"`.
4. `destore proxy` will automatically extract the schmas from all the ELFs you flash to the device and store them in the
   `.destore` directory of the project.
5. Use `destore dump <FLASH_OFFSET> <SIZE>` to dump the records from the flash memory of an attached device. Schema is
   looked up from the
   cache dir.

