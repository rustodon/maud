use std::borrow::IntoCow;
use syntax::ast::{Expr, Ident, Stmt};
use syntax::ext::base::ExtCtxt;
use syntax::parse::token;
use syntax::ptr::P;

use super::parse::Escape;
use maud::escape;

pub struct Renderer<'cx, 's: 'cx, 'o> {
    pub cx: &'cx mut ExtCtxt<'s>,
    stmts: &'o mut Vec<P<Stmt>>,
    w: Ident,
}

impl<'cx, 's: 'cx, 'o> Renderer<'cx, 's, 'o> {
    pub fn with<F>(cx: &'cx mut ExtCtxt<'s>, f: F) -> P<Expr> where
        F: for<'o_> FnOnce(&mut Renderer<'cx, 's, 'o_>)
    {
        let mut stmts = vec![];
        let w = Ident::new(token::intern("w"));
        let cx = {
            let mut render = Renderer {
                cx: cx,
                stmts: &mut stmts,
                w: w,
            };
            f(&mut render);
            render.cx
        };
        quote_expr!(cx, |&: $w: &mut ::std::fmt::Writer| -> ::std::result::Result<(), ::std::fmt::Error> {
            $stmts
            Ok(())
        })
    }

    /// Append a literal pre-escaped string.
    pub fn write(&mut self, s: &str) {
        let w = self.w;
        self.stmts.push(quote_stmt!(self.cx, try!($w.write_str($s))));
    }

    /// Append a literal string, with the specified escaping method.
    pub fn string(&mut self, s: &str, escape: Escape) {
        let s = match escape {
            Escape::None => s.into_cow(),
            Escape::Attr => escape::attribute(s).into_cow(),
            Escape::Body => escape::non_attribute(s).into_cow(),
        };
        self.write(s.as_slice());
    }

    /// Append the result of an expression, with the specified escaping method.
    pub fn splice(&mut self, expr: P<Expr>, escape: Escape) {
        let w = self.w;
        self.stmts.push(match escape {
            Escape::None => quote_stmt!(self.cx, try!(write!($w, "{}", $expr))),
            Escape::Attr =>
                quote_stmt!(self.cx,
                    try!(::maud::rt::escape_attribute($w, |w| write!(w, "{}", $expr)))),
            Escape::Body =>
                quote_stmt!(self.cx,
                    try!(::maud::rt::escape_non_attribute($w, |w| write!(w, "{}", $expr)))),
        });
    }

    pub fn element_open_start(&mut self, name: &str) {
        self.write("<");
        self.write(name);
    }

    pub fn attribute_start(&mut self, name: &str) {
        self.write(" ");
        self.write(name);
        self.write("=\"");
    }

    pub fn attribute_end(&mut self) {
        self.write("\"");
    }

    pub fn element_open_end(&mut self) {
        self.write(">");
    }

    pub fn element_close(&mut self, name: &str) {
        self.write("</");
        self.write(name);
        self.write(">");
    }
}