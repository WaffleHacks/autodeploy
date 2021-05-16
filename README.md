# AutoDeploy

Automatically deploy GitHub repos using webhooks.
Can be configured to deploy every push or every release.
By default it automatically deploy every repository's webhook it receives, but it can be configured to blacklist or whitelist certain repositories.

__NOTE__: the initial deployment is out scope for this tool.

### Configuration
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
