# Define a section for destore and make sure it is not optimized away
SECTIONS
{
  .destore 1 (INFO) :
  {
    KEEP(*(.destore.*));
  }
}