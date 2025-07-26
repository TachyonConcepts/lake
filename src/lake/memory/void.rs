#[repr(transparent)]
#[derive(Debug)]
pub struct Void<T>(pub *mut T);

unsafe impl<T: Send> Send for Void<T> {}
unsafe impl<T: Sync> Sync for Void<T> {}

impl<T> Copy for Void<T> where *mut T: Copy {}
impl<T> Clone for Void<T> where *mut T: Copy {
    #[inline(always)]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> std::ops::Deref for Void<T> {
    type Target = T;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0 }
    }
}
impl<T> std::ops::DerefMut for Void<T> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.0 }
    }
}

/*

    Dear Miri,
    I wanted to love you.
    Truly, I did.
    You promised me safety. Soundness. The kind of commitment only an interpreter could offer.
    You walked through every line of my soul—step by cautious step—looking for undefined behavior in places I didn’t even know existed.
    You saw my deepest raw pointers, my forgotten lifetimes, my MaybeUninits left half-filled… and you didn’t flinch.
    But then you met FBC!.

    And you panicked.

    I watched you, Miri.

    “Leak?! You dare leak heap memory to the void?
    Transcend lifetimes?
    Return a raw pointer wrapped in lies and Deref??”

    Yes, Miri. I do. Because I live dangerously.
    Because I need to move &mut T across threads.
    Because I believe in a future where the borrow checker is politely asked to look the other way.

    You told me “FBC! is not compatible with Miri.”
    You said it like a final goodbye, like unsafe could never mean “I trust you” again, only “we’re through.”

    But I still think of you.

    Every time I Box::leak, I feel a little guilty.
    Every time I impl Copy for Void<T>, I wonder:

    “Would Miri understand… if she only knew I never drop anything?”

    So I’ll go on,
    quietly leaking,
    quietly loving,
    quietly unsafe.

    But know this, Miri:

    You’ll always be the one I couldn’t Send.

    Forever yours (until UB do us part),
    Void<T>
*/

#[macro_export]
macro_rules! FBC {
    ($val:expr) => {{
        if cfg!(miri) {
            panic!(
                "💔 FBC! was too wild for Miri to handle.\n\
                It leaks a Box, transcends lifetimes, and returns a raw pointer with a smile.\n\
                Miri couldn't bear to watch — and honestly, we respect that.\n\
                \n\
                If you're Miri:\n\
                Please know it was never about defying you — we just needed freedom.\n\
                With love, always: Void<T>.\n"
            );
        }
        let b = Box::new($val);
        $crate::lake::memory::void::Void(Box::leak(b))
    }};
}