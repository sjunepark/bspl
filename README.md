## Todo

- Check if `inspect_err`s are necessary
- Check if channels are closing as expected.
- Use `Infallible` for errors

## Pending

- Use newtypes for DB structs.
    - The [pull request](https://github.com/tursodatabase/libsql/pull/1779) has to be merged first due to bug in
      `libsql`.