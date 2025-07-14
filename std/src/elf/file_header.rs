//! Information about an ELF file

/// ELF file identifier
#[repr(C, packed)]
#[derive(Debug)]
pub struct Identifier {
    /// Magic value of file
    pub magic: [u8; 4],
    /// File class
    pub class: u8,
    /// Data encoding
    pub data: u8,
    /// File version
    pub version: u8,
    /// OS/ABI identification
    pub os_abi: u8,
    /// ABI version
    pub abi_version: u8,
    /// Padding bytes
    pub padding: [u8; 6],
    /// Size of identifier
    pub ident_size: u8,
}

/// ELF file header
#[derive(Debug)]
#[repr(C)]
pub struct FileHeader {
    /// File identifier
    pub identifier: Identifier,
    /// Object file type
    pub file_type: u16,
    /// Machine type
    pub machine_type: u16,
    /// Object file version
    pub version: u32,
    /// Entry point address
    pub entry: u64,
    /// Program header offset
    pub phoff: u64,
    /// Section header offset
    pub shoff: u64,
    /// Processor-specific flags
    pub flags: u32,
    /// Elf header size
    pub ehsize: u16,
    /// Size of program header entry
    pub phentsize: u16,
    /// Number of program header entries
    pub phnum: u16,
    /// Size of section header entry
    pub shentsize: u16,
    /// Number of section header entries
    pub shnum: u16,
    /// Section name string table index
    pub shstrndx: u16,
}

impl FileHeader {
    /// Returns the file header at given address, if the magic value present is correct
    pub fn from_addr(addr: usize) -> Option<&'static FileHeader> {
        let header = unsafe { &*(addr as *const FileHeader) };

        if header.identifier.magic == *b"\x7FELF" {
            Some(header)
        } else {
            None
        }
    }
}
