pub(crate) mod downcast {
    /// Code originally from https://github.com/hyperium/http/blob/master/src/convert.rs,
    /// licensed under the following license:
    /*
    Copyright (c) 2017 http-rs authors

    Permission is hereby granted, free of charge, to any
    person obtaining a copy of this software and associated
    documentation files (the "Software"), to deal in the
    Software without restriction, including without
    limitation the rights to use, copy, modify, merge,
    publish, distribute, sublicense, and/or sell copies of
    the Software, and to permit persons to whom the Software
    is furnished to do so, subject to the following
    conditions:

    The above copyright notice and this permission notice
    shall be included in all copies or substantial portions
    of the Software.

    THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF
    ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED
    TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A
    PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT
    SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
    CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
    OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR
    IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
    DEALINGS IN THE SOFTWARE.
    */
    macro_rules! if_downcast_into {
        ($in_ty:ty, $out_ty:ty, $val:ident, $body:expr) => {{
            if std::any::TypeId::of::<$in_ty>() == std::any::TypeId::of::<$out_ty>() {
                // Store the value in an `Option` so we can `take`
                // it after casting to `&mut dyn Any`.
                let mut slot = Some($val);
                // Re-write the `$val` ident with the downcasted value.
                let $val = (&mut slot as &mut dyn std::any::Any)
                    .downcast_mut::<Option<$out_ty>>()
                    .unwrap()
                    .take()
                    .unwrap();
                // Run the $body in scope of the replaced val.
                $body
            }
        }};
    }

    pub(crate) use if_downcast_into;
}
