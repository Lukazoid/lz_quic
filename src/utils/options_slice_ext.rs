use errors::*;

pub trait OptionsSliceExt {
    type InnerItem;

    fn replace_nones<I: IntoIterator<Item = Self::InnerItem>>(
        &mut self,
        replacements: I,
    ) -> Result<()>;
}

impl<'a, T> OptionsSliceExt for &'a mut [Option<T>] {
    type InnerItem = T;
    fn replace_nones<I: IntoIterator<Item = Self::InnerItem>>(
        &mut self,
        replacements: I,
    ) -> Result<()> {
        let mut iter_mut = self.iter_mut().filter(|option| option.is_none());

        let mut replacements_iterator = replacements.into_iter();

        while let Some(to_replace) = iter_mut.next() {
            if let Some(replacement) = replacements_iterator.next() {
                *to_replace = Some(replacement);
            } else {
                bail!(ErrorKind::NotEnoughReplacementValues);
            }
        }

        if let Some(_) = replacements_iterator.next() {
            bail!(ErrorKind::NotEnoughValuesToReplace);
        }

        Ok(())
    }
}
