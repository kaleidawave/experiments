#![allow(unused)]

#[derive(Debug, Clone)]
pub struct MyStruct(pub u8);

#[derive(Debug, Clone)]
pub struct VecWithCustomSize<T, U>(*mut u8, T, T, std::marker::PhantomData<U>);

// **4 the demo**
impl<T: Default, U> VecWithCustomSize<T, U> {
	pub fn new() -> Self {
		Self(unsafe { std::mem::transmute(0usize) }, T::default(), T::default(), Default::default())
	}
}

fn main() {
	dbg!(std::mem::size_of::<Vec<MyStruct>>()); // 24. u64 + u64 + ptr. 8 * 3
	dbg!(std::mem::size_of::<std::ops::Range<u8>>()); // 2. u8 + u8
	
	let mut buckets: [Vec<MyStruct>; 16] = std::array::repeat(Vec::new());
	dbg!(std::mem::size_of_val(&buckets)); // = 384!!!!!
	
	// --- want ---

	assert_eq!(std::mem::size_of::<VecWithCustomSize<u32, MyStruct>>(), 16); // ptr + 2 * 4 = 16
	
	let mut buckets: [VecWithCustomSize<u32, MyStruct>; 16] = std::array::repeat(VecWithCustomSize::new());
	dbg!(std::mem::size_of_val(&buckets)); // = 256, a little better
}
