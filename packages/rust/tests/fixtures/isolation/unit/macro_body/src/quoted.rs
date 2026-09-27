pub fn shape() -> usize {
    1
}

#[cfg(test)]
mod tests {
    #[test]
    fn a_quoted_template_is_not_a_reach() {
        let _tokens = quote::quote! { crate::other::load() };
        let _spanned = quote::quote_spanned! { span => rand::random::<u8>() };
    }

    #[test]
    fn a_vec_of_locals_is_not_a_reach() {
        let _ = vec![super::shape(), super::shape()];
    }
}
