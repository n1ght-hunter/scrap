use super::NewParser;



impl<'a> NewParser<'a> {
    #[inline(always)]
    pub fn get_or_intern(&mut self, name: &str) -> scrap_span::Symbol {
        scrap_span::Symbol(self.lasso.get_or_intern(name))
    }

    #[inline(always)]
    pub fn resolve_symbol(&self, symbol: scrap_span::Symbol) -> &str {
        self.lasso.resolve(&symbol.0)
    }
}