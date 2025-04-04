import argv
import gleam/io
import gleam/list
import gleam/string
import httpee/commands/list as list_command
import httpee/template
import simplifile
import tom

pub fn main() {
  let assert Ok(files) = simplifile.read_directory("./example")
  let request_files =
    list.filter(files, fn(file) { string.ends_with(file, "toml") })

  let templates =
    list.map(request_files, fn(file_name) {
      let assert Ok(file_contents) = simplifile.read("./example/" <> file_name)
      let assert Ok(t) = tom.parse(file_contents)
      template.decode(t, file_name)
    })

  let x =
    list.try_fold(templates, [], fn(acc, v) {
      case v {
        Error(e) -> Error(e)
        Ok(v) -> Ok(list.append(acc, [v]))
      }
    })

  case x {
    Error(err) -> {
      io.debug(err)
      Nil
    }
    Ok(templates) -> list_command.render(templates)
  }
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
