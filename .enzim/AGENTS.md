# enzim Store publishing

Managed by the enzim Apps GitHub App.

Use https://flatpak.enzim.dev/dashboard.html to select a release, submit your
listing and start/track its build. The workflow delegates to an approved enzim
builder pinned by commit. No enzim signing or deployment keys belong here.

For coding agents: read https://flatpak.enzim.dev/agents.md and the machine
contract at https://flatpak.enzim.dev/developer-api/contract. The dashboard can
issue a short-lived repository-scoped agent token. Never put credentials in
issues, manifests, source commits or agent instructions.

Packaging is a self-contained JSON Flatpak manifest. Pin every Git dependency
to a full commit and set disable-submodules: true. Pin downloaded files/archives
by SHA-256. Build scripts run offline. The main source must reference this
repository; use @ENZIM_SOURCE_COMMIT@ for that main source. Declare dependencies as sources; do
not download binaries or dependencies from build commands. Changes need manual
review; successful builds and attestations do not approve publication.

Optional app-specific scalar settings use ENZIM_* environment names. The developer or their agent sets ENZIM_BUILD_CONFIG in their own repository
using GitHub settings or gh secret set. The enzim App has no Secrets permission. Do not pass backend credentials or signing keys. Values
compiled into a desktop app can be extracted after distribution.
