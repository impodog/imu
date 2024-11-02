pub const EOF: char = 0 as char;

/// A queue of 2 chars used store peeked chars
///
/// Only 2 elements are supported, thus is optimized for this usage
#[derive(Debug, Clone, Copy)]
struct PeekQueue {
    first: char,
    second: char,
}

impl PeekQueue {
    /// Creates an empty queue
    fn new() -> Self {
        Self {
            first: EOF,
            second: EOF,
        }
    }

    /// Pushes a new char into the tail of the queue
    /// Calling this function when `self.len() == 2` will discard the first char pushed
    fn push(&mut self, c: char) {
        if self.second == EOF {
            self.second = c;
        } else {
            self.first = self.second;
            self.second = c;
        }
    }

    /// Pops a char from the head of the queue
    /// If the queue is empty, EOF is returned
    fn pop(&mut self) -> char {
        if self.first != EOF {
            std::mem::replace(&mut self.first, EOF)
        } else {
            std::mem::replace(&mut self.second, EOF)
        }
    }

    /// Get the length of the queue, guaranteed to be one of 0, 1, or 2
    fn len(&self) -> usize {
        if self.first == EOF {
            if self.second == EOF {
                0
            } else {
                1
            }
        } else if self.second == EOF {
            1
        } else {
            2
        }
    }

    /// Gets the first element in queue
    /// If the queue is empty, EOF is returned
    fn first(&self) -> char {
        if self.first == EOF {
            self.second
        } else {
            self.first
        }
    }

    /// Gets the second element in queue
    /// If the queue is not full, EOF is returned
    fn second(&self) -> char {
        if self.first == EOF {
            EOF
        } else {
            self.second
        }
    }
}

/// Reads and peeks chars from iterator, main entry for the lexer
///
/// A range of 2 chars peeking is supported
pub struct Reader<I>
where
    I: Iterator<Item = char>,
{
    iter: I,
    cursor: usize,
    queue: PeekQueue,
}

impl<I> Reader<I>
where
    I: Iterator<Item = char>,
{
    /// Creates a new reader over an iterator
    pub fn new(iter: impl IntoIterator<Item = char, IntoIter = I>) -> Self {
        Self {
            iter: iter.into_iter(),
            cursor: 0,
            queue: PeekQueue::new(),
        }
    }

    /// Gets the next char of `self.iter`, moving the cursor forward by the length of the char
    ///
    /// If the iter reaches eof, EOF is returned
    pub fn next_char(&mut self) -> char {
        let ch = self.queue.pop();
        if ch == EOF {
            if let Some(ch) = self.iter.next() {
                self.cursor += ch.len_utf8();
                ch
            } else {
                EOF
            }
        } else {
            self.cursor += ch.len_utf8();
            ch
        }
    }

    /// Similar to `self.next_char()`, but does not return a char
    pub fn advance(&mut self) {
        self.next_char();
    }

    /// Advances the cursor by one char until `f` returns false, discarding chars in between
    pub fn advance_while(&mut self, mut f: impl FnMut(&mut Self) -> bool) {
        while f(self) {
            self.advance();
        }
    }

    /// Returns the cursor pointing to the start of the next unread char
    pub fn cursor(&self) -> usize {
        self.cursor
    }

    /// Peeks the first char from the input stream, without moving the cursor
    pub fn first(&mut self) -> char {
        let first = self.queue.first();
        if first == EOF {
            let next = self.iter.next().unwrap_or(EOF);
            self.queue.push(next);
            next
        } else {
            first
        }
    }

    /// Peeks the second char from the input stream, without moving the cursor
    pub fn second(&mut self) -> char {
        match self.queue.len() {
            0 => {
                let next = self.iter.next().unwrap_or(EOF);
                self.queue.push(next);
                let next = self.iter.next().unwrap_or(EOF);
                self.queue.push(next);
                next
            }
            1 => {
                let next = self.iter.next().unwrap_or(EOF);
                self.queue.push(next);
                next
            }
            2 => self.queue.second(),
            _ => unreachable!(),
        }
    }

    /// Returns `self.cursor() - begin`, useful for length calculation
    pub(crate) fn diff(&self, begin: usize) -> usize {
        self.cursor - begin
    }
}
