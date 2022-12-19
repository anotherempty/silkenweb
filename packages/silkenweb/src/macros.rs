pub use futures_signals::{signal::Signal, signal_vec::SignalVec};
pub use paste::paste;
pub use silkenweb_base::intern_str;
pub use silkenweb_macros::rust_to_html_ident;
pub use wasm_bindgen::{JsCast, JsValue, UnwrapThrowExt};
pub use web_sys;

/// Define a custom html element.
///
/// This will define a struct for an html element, with a method for each
/// attribute. It will also define a struct for the built element. Underscores
/// are converted to dashes in element, attribute, and event names, and raw
/// identifiers have their `r#` prefix stripped.
///
/// The html identifier can be explicitly specified in brackets after the
/// element or attribute name. See `my_explicitly_named_attribute` in the
/// example.
///
/// # Example
///
/// ```no_run
/// # use silkenweb::custom_html_element;
/// use silkenweb::elements::CustomEvent;
///
/// // The types of the dom element and event carry through to the event handler.
/// custom_html_element!(my_html_element = {
///     dom_type: web_sys::HtmlDivElement;
///     attributes {
///         my_attribute: String,
///         my_explicitly_named_attribute("MyExplicitlyNamedAttribute"): String
///     };
///
///     events {
///         my_event: web_sys::MouseEvent
///     };
///
///     custom_events {
///         my_custom_event: CustomEvent<web_sys::HtmlElement>,
///     };
/// });
///
/// let elem: MyHtmlElement = my_html_element()
///     .my_attribute("attribute-value")
///     .on_my_event(|event: web_sys::MouseEvent, target: web_sys::HtmlDivElement| {})
///     .on_my_custom_event(|event: CustomEvent<web_sys::HtmlElement>, target: web_sys::HtmlDivElement| {});
/// ```
#[macro_export]
macro_rules! custom_html_element {
    (
        $(#[$elem_meta:meta])*
        $name:ident $( ($text_name: literal) )? = {
            $($tail:tt)*
        }
    ) => {
        $crate::dom_element!(
            $(#[$elem_meta])*
            $name $( ($text_name) )? = {
                common_attributes = [$crate::elements::HtmlElement, $crate::elements::AriaElement];
                common_events = [$crate::elements::HtmlElementEvents];
                namespace = $crate::node::element::Namespace::Html;
                $($tail)*
            }
        );
    }
}

macro_rules! html_element {
    (
        $(#[$elem_meta:meta])*
        $name:ident $( ($text_name: literal) )? = {
            $($tail:tt)*
        }
    ) => {
        $crate::dom_element!(
            $(#[$elem_meta])*
            $name $( ($text_name) )? = {
                common_attributes = [$crate::elements::HtmlElement, $crate::elements::AriaElement];
                common_events = [$crate::elements::HtmlElementEvents];
                namespace = $crate::node::element::Namespace::Html;
                doc_macro = html_element_doc;
                attribute_doc_macro = html_attribute_doc;
                $($tail)*
            }
        );
    }
}

macro_rules! html_element_doc {
    ($name:expr) => {
        concat!(
            "The HTML [",
            $name,
            "](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/",
            $name,
            ") element"
        )
    };
}

macro_rules! html_attribute_doc {
    ($element:expr, $name:expr) => {
        concat!(
            "The [`",
            $name,
            "`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/",
            $element,
            "#attr-",
            $name,
            ") attribute"
        )
    };
}

macro_rules! svg_element {
    (
        $(#[$elem_meta:meta])*
        $name:ident $( ($text_name: literal) )? = {
            $($tail:tt)*
        }
    ) => {
        $crate::dom_element!(
            $(#[$elem_meta])*
            $name $( ($text_name) )? = {
                common_attributes = [$crate::elements::svg::attributes::Global, $crate::elements::AriaElement];
                common_events = [];
                namespace = $crate::node::element::Namespace::Svg;
                doc_macro = svg_element_doc;
                attribute_doc_macro = svg_attribute_doc;
                $($tail)*
            }
        );
    }
}

macro_rules! svg_element_doc {
    ($name:expr) => {
        concat!(
            "The SVG [`",
            $name,
            "`](https://developer.mozilla.org/en-US/docs/Web/SVG/Element/",
            $name,
            ") element"
        )
    };
}

macro_rules! svg_attribute_doc {
    ($element:expr, $name:expr) => {
        concat!(
            "The SVG [`",
            $name,
            "`](https://developer.mozilla.org/en-US/docs/Web/SVG/Attribute/",
            $name,
            ") attribute"
        )
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! dom_element {
    (
        $(#[$elem_meta:meta])*
        $snake_name:ident ($text_name:expr) = {
            camel_name = $camel_name:ident;
            frozen_name = $frozen_name:ident;
            template_name = $template_name:ident;
            common_attributes = [$($attribute_trait:ty),*];
            common_events = [$($event_trait:ty),*];
            namespace = $namespace:expr;
            $(doc_macro = $doc_macro:ident;)?
            $(attribute_doc_macro = $attr_doc_macro:ident;)?
            dom_type: $elem_type:ty;

            $(attributes { $(
                $(#[$attr_meta:meta])*
                $attr:ident $( ($text_attr:expr) )? : $typ:ty
            ),* $(,)? }; )?

            $(events {
                $(
                    $(#[$event_meta:meta])*
                    $event:ident: $event_type:ty
                ),* $(,)?
            };)?

            $(custom_events { $(
                    $(#[$custom_event_meta:meta])*
                    $custom_event:ident: $custom_event_type:ty
                ),* $(,)?
            };)?
        }
    ) => {
        $(
            #[doc = $doc_macro!($text_name)]
            #[doc = ""]
        )?
        $(#[$elem_meta])*
        pub fn $snake_name<D:$crate::dom::Dom>() -> $camel_name<D> {
            $camel_name::new()
        }

        pub struct $camel_name<Dom: $crate::dom::Dom = $crate::dom::DefaultDom>(
            $crate::node::element::GenericElement<Dom>
        );

        impl<Dom: $crate::dom::Dom> $camel_name<Dom> {
            pub fn new() -> Self {
                Self($crate::node::element::GenericElement::new($namespace, $text_name))
            }

            $crate::attributes![
                $([
                        attribute_parent = $text_name,
                        attribute_doc_macro = $attr_doc_macro
                ])?
                $($($(#[$attr_meta])* pub $attr $( ($text_attr) )?: $typ,)*)?
            ];

            $($crate::events!(
                $elem_type {
                    $(
                        $(#[$event_meta])*
                        pub $event: $event_type
                    ),*
                }
            ); )?

            $($crate::custom_events!(
                $elem_type {
                    $(
                        $(#[$custom_event_meta])*
                        $custom_event: $custom_event_type
                    ),*
                }
            ); )?

            pub fn freeze(self) -> $frozen_name<Dom> {
                $frozen_name(self.0.freeze())
            }
        }

        impl<Dom: $crate::dom::Dom> Default for $camel_name<Dom> {
            fn default() -> Self {
                Self::new()
            }
        }

        impl<Dom, InitParam> $camel_name<$crate::dom::Template<Dom, InitParam>>
        where
            Dom: $crate::dom::InstantiableDom,
            InitParam: 'static
        {
            pub fn on_instantiate(
                self,
                f: impl 'static + Fn($camel_name<Dom>, &InitParam) -> $camel_name<Dom>,
            ) -> Self {
                $camel_name(self.0.on_instantiate(
                    move |elem, param| {
                        f($camel_name(elem), param).0
                    }
                ))
            }
        }

        impl<Dom: $crate::dom::Dom> $crate::node::element::Element for $camel_name<Dom> {
            type DomType = $elem_type;

            fn class<'a, T>(self, class: impl $crate::value::RefSignalOrValue<'a, Item = T>) -> Self
            where
                T: 'a + AsRef<str>
            {
                Self(self.0.class(class))
            }

            fn classes<'a, T, Iter>(
                self,
                classes: impl $crate::value::RefSignalOrValue<'a, Item = Iter>,
            ) -> Self
                where
                    T: 'a + AsRef<str>,
                    Iter: 'a + IntoIterator<Item = T>,
            {
                Self(self.0.classes(classes))
            }

            fn attribute<'a>(
                self,
                name: &str,
                value: impl $crate::value::RefSignalOrValue<'a, Item = impl $crate::attribute::Attribute>
            ) -> Self {
                Self(self.0.attribute(name, value))
            }

            fn effect(self, f: impl ::std::ops::FnOnce(&Self::DomType) + 'static) -> Self {
                Self(self.0.effect(|elem| {
                    f($crate::macros::UnwrapThrowExt::unwrap_throw($crate::macros::JsCast::dyn_ref(elem)))
                }))
            }

            fn effect_signal<T: 'static>(
                self,
                sig: impl $crate::macros::Signal<Item = T> + 'static,
                f: impl Fn(&Self::DomType, T) + Clone + 'static,
            ) -> Self {
                Self(self.0.effect_signal(
                    sig,
                    move |elem, signal| {
                        f(
                            $crate::macros::UnwrapThrowExt::unwrap_throw($crate::macros::JsCast::dyn_ref(elem)),
                            signal,
                        )
                    }
                ))
            }

            fn handle(&self) -> $crate::node::element::ElementHandle<Self::DomType> {
                self.0.handle().cast()
            }

            fn spawn_future(self, future: impl ::std::future::Future<Output = ()> + 'static) -> Self {
                Self(self.0.spawn_future(future))
            }

            fn on(
                self,
                name: &'static str,
                f: impl FnMut($crate::macros::JsValue) + 'static
            ) -> Self {
                Self($crate::node::element::Element::on(self.0, name, f))
            }
        }

        impl<Dom: $crate::dom::Dom> $crate::value::Value for $camel_name<Dom> {}

        impl<Dom: $crate::dom::Dom> From<$camel_name<Dom>> for $crate::node::element::GenericElement<Dom> {
            fn from(elem: $camel_name<Dom>) -> Self {
                elem.0
            }
        }

        impl<Dom: $crate::dom::Dom> From<$camel_name<Dom>>
        for $crate::node::Node<Dom> {
            fn from(elem: $camel_name<Dom>) -> Self {
                elem.0.into()
            }
        }

        $(impl<Dom: $crate::dom::Dom> $attribute_trait for $camel_name<Dom> {})*

        $(
            impl<Dom: $crate::dom::Dom> $event_trait for $camel_name<Dom> {}
        )*

        impl<Dom: $crate::dom::Dom> $crate::elements::ElementEvents for $camel_name<Dom> {}

        pub struct $frozen_name<Dom: $crate::dom::Dom = $crate::dom::DefaultDom>(
            $crate::node::element::FrozenElement<Dom>
        );

        impl<Dom, InitParam> $frozen_name<$crate::dom::Template<Dom, InitParam>>
        where
            Dom: $crate::dom::InstantiableDom,
            InitParam: 'static
        {
            pub fn instantiate(&self, param: &InitParam) -> $camel_name<Dom> {
                $camel_name(self.0.instantiate(param))
            }
        }

        pub type $template_name<Dom, InitParam> =
            $frozen_name<$crate::dom::Template<Dom, InitParam>>;
    };
    (
        $(#[$elem_meta:meta])*
        $name:ident $( ($text_name: literal) )? = {
            common_attributes $($tail:tt)*
        }
    ) => { $crate::macros::paste!{
        $crate::dom_element!(
            $(#[$elem_meta])*
            $name($crate::text_name!($name $( ($text_name) )?)) = {
                camel_name = [< $name:camel >];
                frozen_name = [< Frozen $name:camel >];
                template_name = [< $name:camel Template >];
                common_attributes $($tail)*
            }
        );
    }};
}

/// Add `child` and `text` methods to an html element.
///
/// See [`custom_html_element`] for a complete example of defining an html
/// element.
#[macro_export]
macro_rules! parent_element {
    ($name:ident) => {$crate::macros::paste!{
        impl<Dom: $crate::dom::Dom> $crate::node::element::ParentElement<Dom>
        for [< $name:camel >] <Dom>
        {
            fn text<'a, T>(self, child: impl $crate::value::RefSignalOrValue<'a, Item = T>) -> Self
            where
                T: 'a + AsRef<str> + Into<String>
            {
                Self(self.0.text(child))
            }

            fn child(
                self,
                child: impl $crate::value::SignalOrValue<Item = impl $crate::value::Value + Into<$crate::node::Node<Dom>> + 'static>
            ) -> Self
            {
                Self(self.0.child(child))
            }

            fn optional_child(
                self,
                child: impl $crate::value::SignalOrValue<Item = ::std::option::Option<impl $crate::value::Value + Into<$crate::node::Node<Dom>> + 'static>>
            ) -> Self
            {
                Self(self.0.optional_child(child))
            }

            fn children(self, children: impl IntoIterator<Item = impl Into<$crate::node::Node<Dom>>>) -> Self {
                Self(self.0.children(children))
            }

            fn children_signal(
                self,
                children: impl $crate::macros::SignalVec<Item = impl Into<$crate::node::Node<Dom>>> + 'static,
            ) -> Self {
                Self(self.0.children_signal(children))
            }
        }
    }};
}

/// Implement `ShadowRootParent` for the HTML element
#[macro_export]
macro_rules! shadow_parent_element {
    ($name:ident) => {
        $crate::macros::paste! {
            impl<Dom: $crate::dom::InstantiableDom> $crate::node::element::ShadowRootParent<Dom>
            for [< $name:camel >]<Dom>
            {
                fn attach_shadow_children(
                    self,
                    children: impl IntoIterator<Item = impl Into<$crate::node::Node<Dom>>> + 'static
                ) -> Self {
                    [< $name:camel >] (self.0.attach_shadow_children(children))
                }
            }
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! events {
    ($elem_type:ty {
        $(
            $(#[$event_meta:meta])*
            $visiblity:vis $name:ident: $event_type:ty
        ),* $(,)?
    }) => { $crate::macros::paste!{
        $(
            $(#[$event_meta])*
            $visiblity fn [<on_ $name >] (
                self,
                mut f: impl FnMut($event_type, $elem_type) + 'static
            ) -> Self {
                $crate::node::element::Element::on(
                    self,
                    $crate::text_name_intern!($name),
                    move |js_ev| {
                        use $crate::macros::JsCast;
                        // I *think* we can assume event and event.current_target aren't null
                        let event: $event_type = js_ev.unchecked_into();
                        let target: $elem_type =
                            $crate::macros::UnwrapThrowExt::unwrap_throw(
                                event.current_target()
                            )
                            .unchecked_into();
                        f(event, target);
                    }
                )
            }
        )*
    }};
}

#[doc(hidden)]
#[macro_export]
macro_rules! custom_events {
    ($elem_type:ty {
        $(
            $(#[$event_meta:meta])*
            $name:ident: $event_type:ty
        ),* $(,)?
    }) => { $crate::macros::paste!{
        $(
            $(#[$event_meta])*
            pub fn [<on_ $name>] (
                self,
                mut f: impl FnMut($event_type, $elem_type) + 'static
            ) -> Self {
                $crate::node::element::Element::on(
                    self,
                    $crate::text_name_intern!($name),
                    move |js_ev| {
                        use $crate::macros::JsCast;
                        // I *think* it's safe to assume event and event.current_target aren't null
                        let event: $crate::macros::web_sys::CustomEvent =
                            js_ev.unchecked_into();
                        let target: $elem_type =
                            $crate::macros::UnwrapThrowExt::unwrap_throw(
                                event.current_target()
                            )
                            .unchecked_into();
                        f(event.into(), target);
                    }
                )
            }
        )*
    }};
}

#[doc(hidden)]
#[macro_export]
macro_rules! attributes {
    (
        [
            attribute_parent = $element:expr,
            attribute_doc_macro = $attr_doc_macro:ident
        ]
        $(
            $(#[$attr_meta:meta])*
            $visibility:vis $attr:ident $(($text_attr:expr))? : $typ:ty
        ),* $(,)?
     ) => {
        $(
            $crate::attribute!(
                [
                    attribute_parent = $element,
                    attribute_doc_macro = $attr_doc_macro
                ]
                $(#[$attr_meta])*
                $visibility $attr $(($text_attr))?: $typ
            );
        )*
    };
    (
        $(
            $(#[$attr_meta:meta])*
            $visibility:vis $attr:ident $(($text_attr:expr))? : $typ:ty
        ),* $(,)?
     ) => {
        $(
            $crate::attribute!(
                []
                $(#[$attr_meta])*
                $visibility $attr $(($text_attr))?: $typ
            );
        )*
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! attribute {
    (
        [
            $(
                attribute_parent = $element:expr,
                attribute_doc_macro = $attr_doc_macro:ident
            )?
        ]
        $(#[$attr_meta:meta])*
        $visibility:vis $attr:ident : $typ:ty
    ) => {
        $crate::attribute!(
            [
                $(
                    attribute_parent = $element,
                    attribute_doc_macro = $attr_doc_macro
                )?
            ]
                $(#[$attr_meta])*
            $visibility $attr ($crate::macros::rust_to_html_ident!($attr)): $typ
        );
    };
    (
        [
            $(
                attribute_parent = $element:expr,
                attribute_doc_macro = $attr_doc_macro:ident
            )?
        ]
        $(#[$attr_meta:meta])*
        $visibility:vis $attr:ident ($text_attr:expr): $typ:ty
    ) => {
        $(
            #[doc = $attr_doc_macro!($element, $text_attr)]
            #[doc = ""]
        )?
        $(#[$attr_meta])*
        $visibility fn $attr<'a, T>(
            self,
            value: impl $crate::value::RefSignalOrValue<'a, Item = T>
        ) -> Self
        where
            T: $crate::attribute::AsAttribute<$typ>
        {
            $crate::node::element::Element::attribute(self, $crate::intern_static_str!($text_attr), value)
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! text_name_intern {
    ($($name:tt)*) => {
        $crate::intern_static_str!($crate::text_name!($($name)*))
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! text_name {
    ($name:ident($text_name:literal)) => {
        $text_name
    };
    ($name:ident) => {
        $crate::macros::rust_to_html_ident!($name)
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! intern_static_str {
    ($s:expr) => {{
        ::std::thread_local! {
            static NAME: &'static str = $crate::macros::intern_str($s);
        }

        NAME.with(|name| *name)
    }};
}
