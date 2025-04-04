import argv
import gleam/io
import gleam/list
import gleam/string
import httpee/template
import simplifile
import tom

pub fn main() {
  let assert Ok(files) = simplifile.read_directory("./example")
  let request_files =
    list.filter(files, fn(file) { string.ends_with(file, "toml") })

  let parsed_files =
    list.map(request_files, fn(file_name) {
      let assert Ok(file_contents) = simplifile.read("./example/" <> file_name)
      let assert Ok(t) = tom.parse(file_contents)
      t
    })

  let templates = list.map(parsed_files, fn(toml) { template.decode(toml) })

  io.debug(templates)
}
// let assert Ok(f) = simplifile.read("./example.toml")
// let assert Ok(parsed) = tom.parse(f)
//
// let assert Ok(url) = tom.get_string(parsed, ["request", "url"])
// let assert Ok(body) = tom.get_string(parsed, ["request", "body"])
// let assert Ok(headers) = tom.get_table(parsed, ["request", "headers"])
//
// let args = argv.load().arguments
// io.debug(args)
// io.debug(url)
// io.debug(headers)
// io.debug(body)
// io.println("Hello from httpee!")
