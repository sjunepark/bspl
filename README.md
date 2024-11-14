## Todo

- Create macros for defining newtypes
- Upgrade fake to v3
- Finish data_api
- Migrate sqlite db to postgres
- Parsing HTML only works for one bspl. Need to fix the whole logic. Have to decide the data structure first.
- Check if `inspect_err`s are necessary

## Design decisions

### Newtypes

First, I thought of using newtypes for every domain type would ensure a more safe and reliable application.

However, it turned out that I needed so much code to achieve this.
I've realized that accepting in returning pure primitives from crates or modules is much easier to maintain.
If individual crates or modules have their own specific newtypes,
I have to convert them every time I use them in other modules.

As so, I've decided that I'll just be using primitives instead
and will consider the following
to be the source of truth in terms of validation.

1. The database: The database will have CHECK constraints.
2. Each scraper(if possible): For example, the open-dart api crate should validate the output which it returns.