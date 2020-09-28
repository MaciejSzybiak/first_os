extern crate cpuio;

use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use crate::{println, print};
use lazy_static::lazy_static;
use crate::gdt;
use pic8259_simple::ChainedPics;
use spin;

struct PicInServiceRegister {
    port: cpuio::UnsafePort<u8>,
}

impl PicInServiceRegister {
    pub const unsafe fn new() -> PicInServiceRegister {
        PicInServiceRegister {
            port: cpuio::UnsafePort::new(0x20),
        }
    }

    pub fn read(&mut self) -> u8 {
        unsafe { 
            self.port.write(0x0b);
            self.port.read()
        }
    }
}

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.page_fault.set_handler_fn(page_fault_handler);
        unsafe {
            idt.double_fault.set_handler_fn(double_fault_handler).set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }
        idt[InterruptIndex::Timer.as_usize()].set_handler_fn(timer_interrupt_handler);
        idt[InterruptIndex::Keyboard.as_usize()].set_handler_fn(keyboard_interrupt_handler);
        idt[InterruptIndex::Com2.as_usize()].set_handler_fn(default_interrupt_handler);
        idt[InterruptIndex::Com1.as_usize()].set_handler_fn(default_interrupt_handler);
        idt[InterruptIndex::Lpt2.as_usize()].set_handler_fn(default_interrupt_handler);
        idt[InterruptIndex::Floppy.as_usize()].set_handler_fn(default_interrupt_handler);
        idt[InterruptIndex::Lpt1.as_usize()].set_handler_fn(lpt1_interrupt_handler);
        idt[InterruptIndex::Clock.as_usize()].set_handler_fn(default_interrupt_handler);
        idt[InterruptIndex::Peripherial1.as_usize()].set_handler_fn(default_interrupt_handler);
        idt[InterruptIndex::Peripherial2.as_usize()].set_handler_fn(default_interrupt_handler);
        idt[InterruptIndex::Peripherial3.as_usize()].set_handler_fn(default_interrupt_handler);
        idt[InterruptIndex::Mouse.as_usize()].set_handler_fn(default_interrupt_handler);
        idt[InterruptIndex::Fpu.as_usize()].set_handler_fn(default_interrupt_handler);
        idt[InterruptIndex::PrimaryAta.as_usize()].set_handler_fn(default_interrupt_handler);
        idt[InterruptIndex::SecondaryAta.as_usize()].set_handler_fn(default_interrupt_handler);
        idt
    };
}

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

pub static PICS: spin::Mutex<ChainedPics> =
    spin::Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

static PIC_REGISTER: spin::Mutex<PicInServiceRegister> =
    spin::Mutex::new(unsafe { PicInServiceRegister::new() });

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard,
    Cascade,
    Com2,
    Com1,
    Lpt2,
    Floppy,
    Lpt1,
    Clock,
    Peripherial1,
    Peripherial2,
    Peripherial3,
    Mouse,
    Fpu,
    PrimaryAta,
    SecondaryAta,
}

impl InterruptIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }

    fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}

pub fn init_idt() {
    IDT.load();
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: &mut InterruptStackFrame) {
    println!("EXCEPTION: breakpoint\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: &mut InterruptStackFrame, _error_code: u64) -> ! {
        panic!("EXCEPTION: double fault\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: &mut InterruptStackFrame) {
    unsafe {
        PICS.lock().notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}

extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: &mut InterruptStackFrame) {
    use x86_64::instructions::port::Port;
    use spin::Mutex;
    use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};

    lazy_static! {
        static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> = 
            Mutex::new(Keyboard::new(layouts::Us104Key, ScancodeSet1, HandleControl::Ignore));
    }

    let mut keyboard = KEYBOARD.lock();
    let mut port = Port::new(0x60); //keyboard controller port

    let scan_code: u8 = unsafe { port.read() };
    if let Ok(Some(key_event)) = keyboard.add_byte(scan_code) {
        if let Some(key) = keyboard.process_keyevent(key_event) {
            match key {
                DecodedKey::Unicode(character) => print!("{}", character),
                DecodedKey::RawKey(key) => print!("{:?}", key),
            }
        }
    }

    unsafe {
        PICS.lock().notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
    }
}

extern "x86-interrupt" fn lpt1_interrupt_handler(stack_frame: &mut InterruptStackFrame) {
    let irr = PIC_REGISTER.lock().read();
    if irr & 0x80 != 0 { // ignore spurious interrupts
        println!("LPT1 interrupt:\n{:#?}", stack_frame);
        unsafe{
            PICS.lock().notify_end_of_interrupt(InterruptIndex::Lpt1.as_u8());
        }
    }
}

extern "x86-interrupt" fn default_interrupt_handler(_stack_frame: &mut InterruptStackFrame) {
    let irr = PIC_REGISTER.lock().read();
    println!("Interrupt detected (ID: {})", irr - 1);
    unsafe{
        PICS.lock().notify_end_of_interrupt(PIC_1_OFFSET + irr - 1);
    }
}

use x86_64::structures::idt::PageFaultErrorCode;
use crate::hlt_loop;

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: &mut InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    use x86_64::registers::control::Cr2;

    println!("EXCEPTION: page fault");
    println!("Accessed Address: {:?}", Cr2::read());
    println!("Error code: {:?}", error_code);
    println!("{:#?}", stack_frame);
    hlt_loop();
}

#[test_case]
fn test_breakpoint_exception() {
    //invoke the exception
    x86_64::instructions::interrupts::int3();
}
