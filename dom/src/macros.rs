
#[cfg(feature="profile")]
macro_rules! timed {
    ($label:expr, { $($t:tt)* }) => (
        let t0 = ::std::time::Instant::now();
        let r = $($t)*;
        info!("{}: {:?}", $label, t0.elapsed());
        r
    )
}

#[cfg(not(feature="profile"))]
macro_rules! timed {
    ($label:expr, { $($t:tt)* }) => (
        $($t)*
    )
}

macro_rules! get_or_return {
    (ref $opt:expr) => (
        match $opt {
            Some(ref val) => val,
            None => return
        }
    );
    ($opt:expr, $msg:tt $(,$args:tt)*) => (
        match $opt {
            Some(val) => val,
            None => {
                println!($msg $(,$args)*);
                return;
            }
        }
    );
    ($opt:expr) => (
        match $opt {
            Some(val) => val,
            None => return
        }
    );
    ($opt:expr, $msg:tt $(,$args:tt)*) => (
        match $opt {
            Some(val) => val,
            None => {
                println!($msg $(,$args)*);
                return;
            }
        }
    );
}

macro_rules! parse {
    (@default) => (Default::default());
    (@default = $default:expr) => ($default);
    (@build $name:ident { $($var:ident,)* }) => (Ok($name { $($var,)* }));
    (@build { $($var:ident,)* }) => ();
    (@parse $val:expr) => (Parse::parse($val));
    (@parse $val:expr, $parser:expr) => (($parser)($val));
    (@name $var:ident) => (stringify!($var));
    (@name $var:ident ($name:pat)) => ($name);
    (@ $node:ident, [ var $var:ident $( ($name:pat) )? $(: $ty:ty)? $(= $default:expr)? $(=> $parser:expr)?, $( $rest:tt )* ] [$($list1:tt)*] [$($list2:tt)*] $($args:tt)* ) => (
        parse!(@ $node, [$($rest)*] [$($list1)* $var $( ($name) )? $(: $ty)? $(= $default)? $(=> $parser)?,] [$($list2)*] $($args)*)
    );
    (@ $node:ident, [ anim $var:ident $( ($name:pat) )? $(: $ty:ty)? $(= $default:expr)?, $( $rest:tt )* ] [$($list1:tt)*] [$($list2:tt)*] $($args:tt)* ) => (
        parse!(@ $node, [ $($rest)*] [$($list1)* $var $( ($name) )? $(: $ty)? $(= $default)?,] [$($list2)* $var $( ($name) )?,] $($args)* )
    );
    (@ $node:ident, [_ => $items:ident,] [$($list1:tt)*] [$($list2:tt)*] $($args:tt)*) => (
        parse!(@ $node, [] [$($list1)*] [$($list2)*] $($args)* items=$items)
    );
    (@ $node:ident, [] [$($var:ident $( ($name:pat) )? $(: $ty:ty)? $(= $default:expr)? $(=> $parser:expr)?, )*] [$($var2:ident $( ($name2:pat) )?,)*] $(items=$items:ident)?) => (
        $(
            let mut $var $(: $ty)? = parse!( @default $(= $default)* );
        )*
        $( let mut $items = Vec::new(); )?
        for attribute in $node.attributes() {
            let val = attribute.value();
            match attribute.name() {
                $( parse!(@name $var $( ($name) )?) => $var = parse!(@parse val $(,$parser)? )?, )*
                "style" => {
                    for (key, val) in $crate::util::style_list(val) {
                        match key {
                            $( parse!(@name $var $( ($name) )?) => $var = parse!(@parse val $(,$parser)? )?, )*
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }
        #[allow(unused)]
        for (first, last, n) in crate::first_or_last_node($node.children()) {
            if n.is_element() {
                match n.tag_name().name() {
                    "animate" | "animateColor" => match n.attribute("attributeName").unwrap() {
                        $( parse!(@name $var2 $( ($name2) )?) => $var2.parse_animate_node(&n)?, )*
                        _ => continue,
                    }
                    _ => {}
                }
            }
            $(
                if let Some(item) = parse_node(&n, first, last)? {
                    $items.push(Arc::new(item));
                }
            )?
        }
    );
    (@ $($t:tt)*) => (compile_error!(stringify!($($t)*)));
    ($node:ident => {$($t:tt)*}) => {
        parse!(@ $node, [$($t)*] [] [])
    };
}

// enum_dispatch breaks RLS, so we do it manually
macro_rules! items {
    ($(#[$meta:meta])* pub enum $name:ident { $($($e:pat )|* => $variant:ident($data:ty), )* } { $($other:ident($other_data:ty),)* }) => {
        $( #[$meta] )*
        pub enum $name {
            $( $variant($data), )*
            $( $other($other_data), )*
        }
        impl Tag for $name {
            fn id(&self) -> Option<&str> {
                match *self {
                    $( $name::$variant ( ref tag ) => tag.id(), )*
                    _ => None,
                }
            }
            fn children(&self) -> &[Arc<Item>] {
                match *self {
                    $( $name::$variant ( ref tag ) => tag.children(), )*
                    _ => &[]
                }
            }
        }
        fn parse_element(node: &Node) -> Result<Option<Item>, Error> {
            //println!("<{:?}:{} id={:?}, ...>", node.tag_name().namespace(), node.tag_name().name(), node.attribute("id"));
            let item = match node.tag_name().name() {
                $( $($e )|* => Item::$variant(<$data>::parse_node(node)?), )*
                tag => {
                    println!("unimplemented: {}", tag);
                    return Ok(None);
                }
            };
            Ok(Some(item))
        }
    };
}
