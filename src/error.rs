use std::error::Error;
use std::fmt::{Debug, Display};

pub mod telegram {
    use super::*;

    #[derive(Debug)]
    pub struct CallbackMessageError;

    #[derive(Debug)]
    pub struct CallbackEmptyError;

    impl Error for CallbackMessageError { }
    impl Error for CallbackEmptyError { }

    impl Display for CallbackMessageError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "Сообщение бота больше недоступно.")
        }
    }

    impl Display for CallbackEmptyError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "Был получен пустой callback-запрос.")
        }
    }
}

pub mod serde {
    use super::*;
    // use crate::arenatree::NodeId;

    #[derive(Debug)]
    pub struct FileFormattingError;
    #[derive(Debug)]
    pub struct IndexError<P> { pub index: P }

    impl Error for FileFormattingError { }
    impl<P: Debug + Display> Error for IndexError<P> { }

    impl Display for FileFormattingError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "Неверный формат JSON-файла: недопустимый синтаксис.")
        }
    }

    impl<P: Debug + Display> Display for IndexError<P> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "Нет элемента дерева с таким индексом: {}", self.index)
        }
    }
}
