```shell script
$ cargo run

# should return some data in json format
$ curl -v -H "Content-Type: application/json" -d '{"query":"foo"}'  http://localhost:8080/query


# should return a json with status and message field indicating that `null` could not be deserialized
curl -v -H "Content-Type: application/json" -d '{"query":null}'  http://localhost:8080/query

# should return a custom error object in json format
$ curl -v  http://localhost:8080/error
```