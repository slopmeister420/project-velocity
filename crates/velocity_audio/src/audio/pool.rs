//! _AUDIO_CreatePool.cpp 0x003133D0: the visos audio pools (and the per-type alloc/free pairs at 0x00312170..).
//! the original threads a free list through fixed arrays: allocation pops the head, free pushes back on the
//! head, initial order is index order. handles here carry a generation so a stale id never aliases.

use std::marker::PhantomData;

/// a typed pool handle; 0 is never a valid handle (the original's null pointer).
pub struct Id<T> {
    raw: u32,
    _t: PhantomData<fn() -> T>,
}

impl<T> Id<T> {
    fn new(index: usize, generation: u16) -> Self {
        Self { raw: ((generation as u32) << 16) | (index as u32 + 1), _t: PhantomData }
    }
    pub fn index(self) -> usize {
        (self.raw & 0xFFFF) as usize - 1
    }
    fn generation(self) -> u16 {
        (self.raw >> 16) as u16
    }
    pub fn raw(self) -> u32 {
        self.raw
    }
}

impl<T> Clone for Id<T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T> Copy for Id<T> {}
impl<T> PartialEq for Id<T> {
    fn eq(&self, o: &Self) -> bool {
        self.raw == o.raw
    }
}
impl<T> Eq for Id<T> {}
impl<T> std::hash::Hash for Id<T> {
    fn hash<H: std::hash::Hasher>(&self, h: &mut H) {
        self.raw.hash(h)
    }
}
impl<T> std::fmt::Debug for Id<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Id({}#{})", self.index(), self.generation())
    }
}

struct Slot<T> {
    generation: u16,
    value: Option<T>,
}

/// fixed-capacity object pool with the original's lifo free list.
pub struct Pool<T> {
    slots: Vec<Slot<T>>,
    free: Vec<usize>, // top of stack = head of the original free list
}

impl<T> Pool<T> {
    pub fn new(capacity: usize) -> Self {
        let slots = (0..capacity).map(|_| Slot { generation: 1, value: None }).collect();
        Self { slots, free: (0..capacity).rev().collect() }
    }
    pub fn capacity(&self) -> usize {
        self.slots.len()
    }
    pub fn free_count(&self) -> usize {
        self.free.len()
    }
    pub fn alloc_count(&self) -> usize {
        self.slots.len() - self.free.len()
    }
    pub fn alloc(&mut self, v: T) -> Option<Id<T>> {
        let i = self.free.pop()?;
        let s = &mut self.slots[i];
        s.value = Some(v);
        Some(Id::new(i, s.generation))
    }
    pub fn free(&mut self, id: Id<T>) -> Option<T> {
        let s = self.slots.get_mut(id.index())?;
        if s.generation != id.generation() || s.value.is_none() {
            return None;
        }
        s.generation = s.generation.wrapping_add(1).max(1);
        self.free.push(id.index());
        s.value.take()
    }
    pub fn get(&self, id: Id<T>) -> Option<&T> {
        let s = self.slots.get(id.index())?;
        if s.generation == id.generation() { s.value.as_ref() } else { None }
    }
    pub fn get_mut(&mut self, id: Id<T>) -> Option<&mut T> {
        let s = self.slots.get_mut(id.index())?;
        if s.generation == id.generation() { s.value.as_mut() } else { None }
    }
    pub fn contains(&self, id: Id<T>) -> bool {
        self.get(id).is_some()
    }
    /// live ids in index order (the original's `for j < count` walks over the backing array).
    pub fn ids(&self) -> Vec<Id<T>> {
        self.slots
            .iter()
            .enumerate()
            .filter(|(_, s)| s.value.is_some())
            .map(|(i, s)| Id::new(i, s.generation))
            .collect()
    }
    /// _AUDIO_ResetPool.cpp 0x00314920: every object back on the free list in index order.
    pub fn reset(&mut self) {
        for s in &mut self.slots {
            if s.value.take().is_some() {
                s.generation = s.generation.wrapping_add(1).max(1);
            }
        }
        self.free = (0..self.slots.len()).rev().collect();
    }
}

/// the group struct sub-allocators (`_AUDIO_AllocGroupEvents` 0x00312B30 etc.): a bump allocator that can
/// only give back its most recent allocation. only the accounting matters to the port.
#[derive(Clone, Debug)]
pub struct StackPool {
    pub capacity: u32,
    pub remaining: u32,
    last: u32,
    top: u32,
}

impl StackPool {
    pub fn new(capacity: u32) -> Self {
        Self { capacity, remaining: capacity, last: 0, top: 0 }
    }
    /// returns the start offset, or None when there is not enough left.
    pub fn alloc(&mut self, n: u32) -> Option<u32> {
        if n > self.remaining {
            return None;
        }
        let at = self.top;
        self.top += n;
        self.remaining -= n;
        self.last = n;
        Some(at)
    }
    /// only the latest allocation is returned to the pool; anything else leaks like the original.
    pub fn free(&mut self, at: u32) {
        if at + self.last == self.top {
            self.top = at;
            self.remaining += self.last;
            self.last = 0;
        }
    }
    pub fn reset(&mut self) {
        *self = Self::new(self.capacity);
    }
}
