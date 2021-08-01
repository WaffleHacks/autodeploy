This project is no longer receiving updates and has been superseded by [WaffleMaker](https://github.com/WaffleHacks/wafflemaker).

# AutoDeploy

Automatically deploy GitHub repos using webhooks.
Can be configured to deploy every push or every release.
By default it automatically deploy every repository's webhook it receives, but it can be configured to blacklist or whitelist certain repositories.

At the moment, only GitHub repositories are supported as that is what we use for all our services.

__NOTE__: the initial deployment is out scope for this tool.

## Configuration
There are two things that must be configured for `autodeploy` to work: the server to deploy to and the repositories to deploy.
Their respective configurations are documented below.

### Server
Configuration is done using the `config.toml` file [(example)](./config.example.toml).
The following items are configurable:
- Listen address
- Webhook secret
- Deployable events
  - push to branch
  - release created
- Repositories deployed
  - whitelist or blacklist
  - accepted repositories
  
### Repository
Configuration is read from the `autodeploy.toml` located at the root of the repository [(example)](./autodeploy.example.toml).
The currently supported operations are:
- run a command
- copy a file

More operations may be added in the future.
