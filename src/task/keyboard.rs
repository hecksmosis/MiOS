use crate::{commands::handle_command, print, println, vga_buffer::del_char};
use alloc::string::String;
use conquer_once::spin::OnceCell;
use core::{
    pin::Pin,
    task::{Context, Poll},
};
use crossbeam_queue::ArrayQueue;
use futures_util::{task::AtomicWaker, Stream, StreamExt};
use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};
use vga::vga::VideoMode;

static WAKER: AtomicWaker = AtomicWaker::new();
static SCANCODE_QUEUE: OnceCell<ArrayQueue<u8>> = OnceCell::uninit();

pub struct AppContext {
    pub prompt: String,
    pub command_cache: String,
    pub mode: VideoMode,
}

impl AppContext {
    pub fn new() -> Self {
        AppContext {
            prompt: String::from("mios> "),
            command_cache: String::new(),
            mode: VideoMode::Mode80x25,
        }
    }
}

pub struct ScancodeStream {
    _private: (),
}

impl ScancodeStream {
    pub fn new() -> Self {
        SCANCODE_QUEUE
            .try_init_once(|| ArrayQueue::new(100))
            .expect("ScancodeStream::new should only be called once");
        ScancodeStream { _private: () }
    }
}

impl Stream for ScancodeStream {
    type Item = u8;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<u8>> {
        let queue = SCANCODE_QUEUE
            .try_get()
            .expect("scancode queue not initialized");

        if let Ok(scancode) = queue.pop() {
            return Poll::Ready(Some(scancode));
        }

        WAKER.register(&cx.waker());
        match queue.pop() {
            Ok(scancode) => {
                WAKER.take();
                Poll::Ready(Some(scancode))
            }
            Err(crossbeam_queue::PopError) => Poll::Pending,
        }
    }
}

pub(crate) fn add_scancode(scancode: u8) {
    if let Ok(queue) = SCANCODE_QUEUE.try_get() {
        if let Err(_) = queue.push(scancode) {
            println!("WARNING: scancode queue full; dropping keyboard input");
        } else {
            WAKER.wake();
        }
    } else {
        println!("WARNING: scancode queue uninitialized");
    }
}

pub async fn handle_keyboard() {
    let mut scancodes = ScancodeStream::new();
    let mut keyboard = Keyboard::new(layouts::Us104Key, ScancodeSet1, HandleControl::Ignore);

    let mut context = AppContext::new();

    while let Some(scancode) = scancodes.next().await {
        if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
            if let Some(key) = keyboard.process_keyevent(key_event) {
                match &context.mode {
                    VideoMode::Mode80x25 => handle_key(key, &mut context),
                    VideoMode::Mode640x480x16 => {}
                    _ => {}
                }
            }
        }
    }
}

fn handle_key(key: DecodedKey, context: &mut AppContext) {
    match key {
        DecodedKey::Unicode(character) if character == '\n' => handle_command(context),
        // handle delete key
        DecodedKey::Unicode(character) if character == '\u{8}' => {
            if context.command_cache.len() > 0 {
                del_char();
                context.command_cache.pop();
            }
        }
        DecodedKey::Unicode(character) => handle_character(&mut context.command_cache, character),
        DecodedKey::RawKey(key) => print!("{:?}", key),
    }
}

pub fn handle_character(command_cache: &mut String, character: char) {
    print!("{}", character);
    command_cache.push(character);
}
