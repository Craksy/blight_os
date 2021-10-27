use conquer_once::spin::OnceCell;
use core::convert::TryInto;
use core::task::Poll;
use crossbeam_queue::ArrayQueue;
use futures_util::{task::AtomicWaker, Stream, StreamExt};

static WAKER: AtomicWaker = AtomicWaker::new();

use crate::keyboard::{decode, Key, KeyboardEvent::*};
use crate::{print, println};

static SCANCODE_QUEUE: OnceCell<ArrayQueue<u8>> = OnceCell::uninit();

pub(crate) fn add_scancode(scancode: u8) {
    if let Ok(queue) = SCANCODE_QUEUE.try_get() {
        if let Err(_) = queue.push(scancode) {
            println!("WARNING. Queue full");
        } else {
            WAKER.wake();
        }
    } else {
        println!("WARNING! Queue not initialized");
    }
}

pub struct ScancodeStream {
    _private: (),
}

impl ScancodeStream {
    pub fn new() -> Self {
        SCANCODE_QUEUE
            .try_init_once(|| ArrayQueue::new(150))
            .expect("Only init queue once");
        Self { _private: () }
    }
}

impl Stream for ScancodeStream {
    type Item = u8;

    fn poll_next(
        self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Option<Self::Item>> {
        let queue = SCANCODE_QUEUE.try_get().expect("Stream not initialized");

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

pub async fn handle_keypresses() {
    let mut scancode_stream = ScancodeStream::new();

    while let Some(scancode) = scancode_stream.next().await {
        if let Some(Make(key)) = decode(scancode) {
            let r: Result<char, ()> = key.try_into();
            if let Ok(character) = r {
                print!("{}", character);
            } else {
                match key {
                    Key::Keypad2 => {
                        crate::vga_buffer::scroll_buffer(-1);
                    }
                    Key::Keypad8 => {
                        crate::vga_buffer::scroll_buffer(1);
                    }
                    _ => print!("{}", scancode),
                }
            }
        }
    }
}
