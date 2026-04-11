#[derive(Default)]
enum State {
    #[default]
    Awaiting,
    Found,
    Seen,
}

/// ```rust
/// const BREAK: u8 = 5;
/// let items: Vec<u8> = vec![1, 3, BREAK, 6];
/// let mut iter = boundary_iterator::BounaryIterator::new(items.into_iter(), BREAK);
/// assert_eq!(iter.by_ref().collect::<Vec<_>>(), &[1, 3]);
/// assert_eq!(iter.expect_item(), Ok(()));
/// assert_eq!(iter.by_ref().collect::<Vec<_>>(), &[6]);
/// ```
pub struct BounaryIterator<I: Iterator> {
    iterator: I,
    boundary: I::Item,
    state: State,
}

impl<I: Iterator> BounaryIterator<I>
where
    I::Item: PartialEq,
{
    // TODO into iter?
    pub fn new(iterator: I, boundary: I::Item) -> Self {
        Self {
            iterator,
            boundary,
            state: State::default(),
        }
    }

    pub fn expect_item(&mut self) -> Result<(), ()> {
        if let State::Found = self.state {
            self.state = State::Seen;
            Ok(())
        } else if let Some(next) = self.iterator.next() {
            if next == self.boundary {
                self.state = State::Seen;
                Ok(())
            } else {
                Err(())
            }
        } else {
            Err(())
        }
    }

    pub fn next_expect(&mut self) -> Option<I::Item> {
        self.iterator.next()
    }
}

impl<I: Iterator> Iterator for BounaryIterator<I>
where
    I::Item: PartialEq,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        if let State::Found = self.state {
            None
        } else {
            let item = self.iterator.next()?;
            if let State::Awaiting = self.state {
                if item == self.boundary {
                    self.state = State::Found;
                    None
                } else {
                    Some(item)
                }
            } else {
                Some(item)
            }
        }
    }
}
