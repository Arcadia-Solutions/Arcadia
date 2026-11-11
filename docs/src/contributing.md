# Contributing

First, thanks for considering contributing to Arcadia!

## Contributing Process

Whether you want to add a new feature or fix an existing issue, it needs to be done on your own branch:

1. Fork this repository
2. Clone it locally on your computer
3. create a new branch `feature-name` or `bug-name-fix` (with the proper name)
4. open a pull request when your contribution is done

If you are unsure about what/how to do something, don't hesitate to [open a discussion](https://github.com/Arcadia-Solutions/arcadia/discussions) or [an issue](https://github.com/Arcadia-Solutions/arcadia/issues) about the topic.

You can also hop on the [Discord server](https://discord.gg/amYWVk7pS3) to chat with other devs and the community.

## Finding Contributions to Make

Arcadia has [boards](https://github.com/Arcadia-Solutions/arcadia/projects) to track the existing issues and features that need to be worked on. Feel free to claim one that isn't claimed yet before starting to work on it.

To claim a github issue, simply leave a comment on it saying that you are working on it.

You can also search for `TODO`s in the code and pick one of those tasks. If you decide to do this, please open an issue first and claim it before working on the task.

## Backend Development Notes

- If you make changes to/add sql queries with `sqlx`, you need to run `cargo sqlx prepare --workspace` at the repository root before committing your changes. It reads the database from the `DATABASE_URL` environment variable (see [Developer Setup](dev-setup.md)), not from `config.yml`. This command will generate some files that allow the queries to be tested without a database running. Our CI pipeline relies on that, and will fail if the command hasn't been ran. You can setup a [git pre-commit hook](https://www.slingacademy.com/article/git-pre-commit-hook-a-practical-guide-with-examples/) if you want.

- For better code quality, we use [clippy](https://github.com/rust-lang/rust-clippy) in our CI pipeline. You can set your editor to run `cargo clippy` instead of `cargo check` (on file save, etc.).
