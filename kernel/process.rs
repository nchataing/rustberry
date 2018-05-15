use alloc::String;
use memory;
use system_control;
use goblin::elf32;
use drivers::mmio;

#[derive(Debug, Clone)]
#[repr(C)]
pub struct RegisterContext
{
    pub r0: u32,
    pub r1: u32,
    pub r2: u32,
    pub r3: u32,
    pub r4: u32,
    pub r5: u32,
    pub r6: u32,
    pub r7: u32,
    pub r8: u32,
    pub r9: u32,
    pub r10: u32,
    pub r11: u32,
    pub r12: u32,
    pub sp: *const u32,
    pub lr: *const u32,
    pub pc: *const u32,
    pub psr: u32,
}

impl RegisterContext
{
    fn new() -> RegisterContext
    {
        RegisterContext { r0: 0, r1: 0, r2: 0, r3: 0, r4: 0, r5: 0, r6: 0,
            r7: 0, r8: 0, r9: 0, r10: 0, r11: 0, r12: 0, sp: 0 as *const u32,
            lr: 0 as *const u32, pc: 0 as *const u32,
            psr: system_control::ProcessorMode::User as u32 }
    }
}

#[derive(PartialEq, Eq)]
pub enum ProcessState
{
    Running,
    Runnable,
    BlockedWriting,
    BlockedReading,
    WaitingChildren,
    Zombie
}

pub struct Process
{
    regs: RegisterContext,
    state: ProcessState,
    name: String,
    memory_map: memory::application_map::ApplicationMap,
}

#[derive(Debug)]
pub enum ElfError
{
    FileTooSmall,
    InvalidMagicNumber,
    InvalidClass,
    InvalidDataEncoding,
    NotExecutable,
    InvalidArchitecture,
    InvalidVersion,
    AppMapError(memory::application_map::AppMapError),
}

impl From<memory::application_map::AppMapError> for ElfError
{
    fn from(err: memory::application_map::AppMapError) -> ElfError
    {
        ElfError::AppMapError(err)
    }
}

impl Process
{
    pub fn new(name: String, elf_file: &[u8]) -> Result<Process, ElfError>
    {
        let mut process = Process { regs: RegisterContext::new(),
            state: ProcessState::Runnable, name,
            memory_map: memory::application_map::ApplicationMap::new() };

        process.load_elf(elf_file)?;
        Ok(process)
    }

    pub fn save_context(&mut self, active_ctx: &RegisterContext)
    {
        assert!(self.state == ProcessState::Running);
        self.regs = active_ctx.clone();
        self.state = ProcessState::Runnable;
    }

    pub fn restore_context(&mut self, active_ctx: &mut RegisterContext)
    {
        assert!(self.state == ProcessState::Runnable);
        *active_ctx = self.regs.clone();
        self.memory_map.activate();
        self.state = ProcessState::Running;
    }

    fn load_elf(&mut self, file_content: &[u8]) -> Result<(), ElfError>
    {
        if file_content.len() < 58
        {
            return Err(ElfError::FileTooSmall);
        }
        let elf_header = elf32::header::Header::from_bytes(array_ref![file_content, 0, 52]);
        if array_ref![elf_header.e_ident, 0, 4] != elf32::header::ELFMAG
        {
            return Err(ElfError::InvalidMagicNumber);
        }
        if elf_header.e_ident[elf32::header::EI_CLASS] != elf32::header::ELFCLASS32
        {
            return Err(ElfError::InvalidClass);
        }
        if elf_header.e_ident[elf32::header::EI_DATA] != elf32::header::ELFDATA2LSB
        {
            return Err(ElfError::InvalidDataEncoding);
        }
        if elf_header.e_type != elf32::header::ET_EXEC
        {
            return Err(ElfError::NotExecutable);
        }
        if elf_header.e_machine != elf32::header::EM_ARM
        {
            return Err(ElfError::InvalidArchitecture);
        }
        if elf_header.e_version != 1
        {
            return Err(ElfError::InvalidVersion);
        }

        let entry_point = elf_header.e_entry;
        let prgm_header_tbl = elf_header.e_phoff as usize;
        let prgm_header_entry_size = elf_header.e_phentsize as usize;
        let nb_prgm_header_entry = elf_header.e_phnum as usize;

        if file_content.len() < prgm_header_tbl + nb_prgm_header_entry * prgm_header_entry_size
        {
            return Err(ElfError::FileTooSmall);
        }

        for entry in 0 .. nb_prgm_header_entry
        {
            let entry_offset = prgm_header_tbl + entry * prgm_header_entry_size;

            let prgm_header_entry = unsafe {
                &*(file_content.as_ptr().offset(entry_offset as isize)
                as *mut elf32::program_header::ProgramHeader) };

            if prgm_header_entry.p_type != elf32::program_header::PT_LOAD
            {
                continue;
            }

            let vaddr = prgm_header_entry.p_vaddr as usize;
            let mem_size = prgm_header_entry.p_memsz as usize;
            let file_offset = prgm_header_entry.p_offset as usize;
            let file_size = prgm_header_entry.p_filesz as usize;
            let flags = prgm_header_entry.p_flags;

            // Reserve application program pages and check that all code remain
            // in the range 0x8000_0000 .. 0x9FFF_FFFF.
            let vpage = vaddr / memory::PAGE_SIZE;
            for page in 0 .. mem_size / memory::PAGE_SIZE
            {
                self.memory_map.register_prgm_page(memory::PageId(vpage+page),
                    flags & elf32::program_header::PF_X != 0,
                    flags & elf32::program_header::PF_W != 0)?;
            }
            self.memory_map.activate();

            if file_content.len() < file_offset + file_size
            {
                return Err(ElfError::FileTooSmall);
            }

            for offset in (0 .. mem_size).step_by(4)
            {
                unsafe
                {
                    if offset < file_size
                    {
                        let file_pos = (file_offset + offset) as isize;
                        mmio::write((vaddr + offset) as *mut u32,
                            *(file_content.as_ptr().offset(file_pos) as *const u32));
                    }
                    else
                    {
                        mmio::write((vaddr + offset) as *mut u32, 0);
                    }
                }
            }
        }

        // We have updated instructions at their physical address so we must
        // flush instruction caches.
        memory::cache::invalidate_instr_cache();
        memory::cache::invalidate_branch_predictor();
        mmio::sync_barrier();
        mmio::instr_barrier();

        self.regs.pc = entry_point as *const u32;

        Ok(())
    }
}
