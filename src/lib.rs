//! A formatted and aligned table printer written in rust
use std::io::{stdout, Write, Error, ErrorKind};
use std::fmt;
use std::str;
use std::string::ToString;

#[cfg(any(unix, macos))]
static LINEFEED: &'static [u8] = b"\n";
#[cfg(windows)]
static LINEFEED: &'static [u8] = b"\r\n";

/// A type representing a row in a table
pub type Row = Vec<String>;

/// A Struct representing a printable table
#[derive(Clone, Debug)]
pub struct Table {
	num_cols: usize,
	titles: Vec<String>,
	rows: Vec<Row>,
	col_sep: char,
	line_sep: char,
	sep_cross: char
}

impl Table {
	/// Create a new table with the number of columns equals to the length of `titles`
	pub fn new(titles: Vec<String>) -> Table {
		let n = titles.len();
		return Table {
			num_cols: n,
			titles: titles, 
			rows: Vec::new(),
			col_sep: '|',
			line_sep: '-',
			sep_cross: '+'
		};
	}
	
	/// Change separators
	/// 
	/// `col` is the column separator
	/// `line` is the line separator
	/// `cross` is a special separator used when line and collumn separators meet
	/// Default separators used are '|', '-' and '+'
	pub fn separators(&mut self, col: char, line: char, cross: char) {
		self.col_sep = col;
		self.line_sep = line;
		self.sep_cross = cross;
	}
	
	/// Get the number of column
	pub fn get_column_num(&self) -> usize {
		return self.num_cols;
	}
	
	/// Get the number of rows
	pub fn len(&self) -> usize {
		return self.rows.len();
	}
	
	/// Get a mutable reference to a row
	pub fn get_mut_row(&mut self, row: usize) -> &mut Row {
		return &mut self.rows[row];
	}
	
	/// Get an immutable reference to a row
	pub fn get_row(&self, row: usize) -> &Row {
		return &self.rows[row];
	}
	
	/// Append a row in the table, transferring ownership of this row to the table
	/// and returning a mutable reference to the row
	pub fn add_row(&mut self, row: Row) -> Result<&mut Row, &str> {
		if row.len() != self.num_cols {
			return Err("Row does not have the proper number of column");
		}
		self.rows.push(row);
		let l = self.rows.len()-1;
		return Ok(self.get_mut_row(l));
	}
	
	/// Append an empty row in the table. Return a mutable reference to this new row.
	pub fn add_empty_row(&mut self) -> Result<&mut Row, &str> {
		let n = self.num_cols;
		return Ok(try!(self.add_row(vec!["".to_string(); n])));	
	}
	
	/// Modify a single element in the table
	pub fn set_element<T: ToString>(&mut self, element: T, column: usize, row: usize) -> Result<(), &str> {
		if column >= self.num_cols {
			return Err("Column index is higher than expected");
		}
		if row > self.rows.len() {
			return Err("Row index is higher than contained number of rows");
		}
		let rowline = self.get_mut_row(row);
		rowline[column] = element.to_string();
		return Ok(());
	}
	
	/// Remove a row. Silently skip if row with index `row` does not exist
	pub fn remove_row(&mut self, row: usize) {
		if row < self.rows.len() {
			self.rows.remove(row);
		}
	}
	
	fn get_col_width(&self, col_idx: usize) -> Result<usize, &str> {
		if col_idx >= self.num_cols {
			return Err("Column index is too high");
		}
		let mut width = self.titles[col_idx].len();
		for r in &self.rows {
			let l = r[col_idx].len();
			if l > width {
				width = l;
			}
		}
		return Ok(width);
	}
	
	fn print_line_separator<T: Write>(&self, out: &mut T, col_width: &[usize]) -> Result<(), Error> {
		try!(out.write_all(self.sep_cross.to_string().as_bytes()));
		for i in 0..self.num_cols {
			for _ in 0..(col_width[i] + 2) {
				try!(out.write_all(self.line_sep.to_string().as_bytes()));
			}
			try!(out.write_all(self.sep_cross.to_string().as_bytes()));
		}
		return out.write_all(LINEFEED);
	}
	
	fn print_line<T: Write>(&self, out: &mut T, line: &[String], col_width: &[usize]) -> Result<(), Error> {
		try!(out.write_all(self.col_sep.to_string().as_bytes()));
		for i in 0..self.num_cols {
			try!(out.write_all(b" "));
			try!(out.write_all(line[i].as_bytes()));
			try!(out.write_all(b" "));
			for _ in 0..(col_width[i] - line[i].len()) {
				try!(out.write_all(b" "));
			}
			try!(out.write_all(self.col_sep.to_string().as_bytes()));
		}
		return out.write_all(LINEFEED);
	}
	
	/// Print the table to `out`
	pub fn print<T: Write>(&self, out: &mut T) -> Result<(), Error> {
		// Compute columns width
		let mut col_width = vec![0usize; self.num_cols];
		for i in 0..self.num_cols {
			col_width[i] = self.get_col_width(i).unwrap();
		}
		// Print titles line
		try!(self.print_line_separator(out, &col_width));
		try!(self.print_line(out, &self.titles, &col_width));
		try!(self.print_line_separator(out, &col_width));
		// Print rows
		for r in &self.rows {
			try!(self.print_line(out, r, &col_width));
			try!(self.print_line_separator(out, &col_width));
		}
		return out.flush();
	}
	
	/// Print the table to standard output
	/// # Panic
	/// Panic if writing to standard output fails
	pub fn printstd(&self) {
		self.print(&mut stdout())
			.ok()
			.expect("Cannot print table to standard output");
	}
}

impl fmt::Display for Table {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
		let mut writer =  StringWriter::new();
		if let Err(_) = self.print(&mut writer) {
			return Err(fmt::Error)
		}
		return fmt.write_str(writer.as_string());
	}
}

/// Internal utility for writing data into a string
struct StringWriter {
	string: String
}

impl StringWriter {
	/// Create a new `StringWriter`
	fn new() -> StringWriter {
		return StringWriter{string: String::new()};
	}
	
	/// Return a reference to the internally written `String`
	fn as_string(&self) -> &String {
		return &self.string;
	}
}

impl Write for StringWriter {
	fn write(&mut self, data: &[u8]) -> Result<usize, Error> {
		let string = match str::from_utf8(data) {
			Ok(s) => s,
			Err(e) => return Err(Error::new(ErrorKind::Other, format!("Cannot decode utf8 string : {}", e)))
		};
		self.string.push_str(string);
		return Ok(data.len());
	}
	
	fn flush(&mut self) -> Result<(), Error> {
		return Ok(());
	}
}

/// Create a table with column's titles from arguments.
///
/// The table can also be initialized with some values.
/// All the arguments used for titles and elements must implement the `std::string::ToString` trait
/// # Syntax
/// table!([Title1, Title2, ...], [Element1_ row1, Element2_ row1, ...], [Element1_row2, ...], ...);
/// # Panic
/// May panic if some rows could not be inserted
/// # Example
/// ```
/// # #[macro_use] extern crate tabprint;
/// # fn main() {
/// // Create an empty table with titles :
/// let tab = table!(["Title1", "Title2", "Title3"]);
/// // Create a table initialized with some rows :
/// let tab = table!(["Title1", "Title2", "Title3"],
/// 				 ["Element1", "Element2", "Element3"],
/// 				 [1, 2, 3]
/// 				 );
/// # drop(tab);
/// # }
/// ```
#[macro_export]
macro_rules! table {
	([$($title: expr), *]) => ($crate::Table::new(vec![$($title.to_string()), *]));
	(
		[$($title: expr), *], $([$($key:expr), *]), *
	) => (
		{
			let mut tab = table!([$($title), *]);
			$(
				let row = vec![$($key.to_string()), *];
				if let Err(e) = tab.add_row(row) {
					panic!("Cannot create table from : {}", e);
				}
			)*
			tab
		}
	)
}

/// Create a table with `table!` macro, print it to standard output, then return this table for future usage.
/// 
/// The syntax is the same that the one for the `table!` macro
/// # Panic
/// May panic if some rows could not be inserted
#[macro_export]
macro_rules! ptable {
	([$($title: expr), *]) => (
		{
			let tab = table!([$($title), *]);
			tab.printstd();
			tab
		}
	);
	(
		[$($title: expr), *], $([$($key:expr), *]), *
	) => (
		{
			let tab = table!([$($title), *], $([$($key), *]), *);
			tab.printstd();
			tab
		}
	)
}