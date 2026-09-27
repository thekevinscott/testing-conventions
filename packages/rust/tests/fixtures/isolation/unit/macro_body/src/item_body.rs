pub fn helper() -> usize {
    2
}

#[cfg(test)]
mod tests {
    // A macro whose token body is a sequence of items, not expressions: the `use` is
    // written here and lands here, so the import rule must see it.
    with_setup! {
        use crate::other::Thing;

        #[test]
        fn uses_the_import() {
            let _ = Thing;
        }
    }
}
