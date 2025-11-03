use std::io::Read;
use std::path::Path;

use ecow::EcoString;
use rustc_hash::FxHashSet;

use crate::errors::ULogError;
use crate::parser::{MmapReader, SliceableReader, ULogParser};

pub struct ULogParserBuilder<R> {
    reader: R,
    include_header: bool,
    include_timestamp: bool,
    include_padding: bool,
    allowed_subscription_names: Option<FxHashSet<EcoString>>,
}

impl<R: Read + SliceableReader> ULogParserBuilder<R> {
    // Start the builder with a mandatory reader
    #[must_use]
    pub fn new(reader: R) -> Self {
        ULogParserBuilder {
            reader,
            include_header: false,
            include_timestamp: false,
            include_padding: false,
            allowed_subscription_names: None,
        }
    }

    #[must_use]
    pub fn include_header(mut self, include: bool) -> Self {
        self.include_header = include;
        self
    }

    #[must_use]
    pub fn include_timestamp(mut self, include: bool) -> Self {
        self.include_timestamp = include;
        self
    }

    #[must_use]
    pub fn include_padding(mut self, include: bool) -> Self {
        self.include_padding = include;
        self
    }

    /// Sets the list of `LoggedData` messages that the parser will return.
    ///
    /// By default, all `LoggedData` messages will be returned, which incurs extra parsing cost.
    ///
    /// Specifying only the required messages in this allow list can greatly improve parser performance.
    ///
    /// Any `LoggedData` messages not included in this allow list will be emitted as raw bytes in a
    /// `UlogMessage::Ignored` variant, so no messages are lost.
    ///
    /// # Parameters
    /// - `subs`: An iterable collection of string-like items representing the names of `LoggedData` messages
    ///   to be parsed fully and returned.
    #[must_use]
    pub fn set_subscription_allow_list<I, S>(mut self, subs: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<EcoString>,
    {
        self.allowed_subscription_names = Some(subs.into_iter().map(|s| s.into()).collect());
        self
    }

    // Final method to build the `ULogParser`
    pub fn build(self) -> Result<ULogParser<R>, ULogError> {
        let result = ULogParser::new(self.reader);

        match result {
            Ok(mut parser) => {
                parser.include_header = self.include_header;
                parser.include_timestamp = self.include_timestamp;
                parser.include_padding = self.include_padding;

                if let Some(allowed_subscr) = self.allowed_subscription_names {
                    parser.set_allowed_subscription_names(allowed_subscr);
                }

                Ok(parser)
            }
            Err(err) => Err(err),
        }
    }
}

impl ULogParserBuilder<MmapReader> {
    /// Creates a new builder from a file path using memory-mapped I/O.
    /// This enables zero-copy parsing for better performance with large files.
    ///
    /// # Example
    /// ```no_run
    /// use yule_log::builder::ULogParserBuilder;
    ///
    /// let parser = ULogParserBuilder::from_file("sample.ulg")?
    ///     .include_header(true)
    ///     .build()?;
    /// # Ok::<(), yule_log::errors::ULogError>(())
    /// ```
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, ULogError> {
        use memmap2::Mmap;
        use std::fs::File;

        let file = File::open(path)?;
        let mmap = unsafe { Mmap::map(&file)? };
        let reader = MmapReader::new(mmap);

        Ok(ULogParserBuilder {
            reader,
            include_header: false,
            include_timestamp: false,
            include_padding: false,
            allowed_subscription_names: None,
        })
    }
}
