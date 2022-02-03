+++
title = "About Chef Habitat"
aliases = ["/habitat/reference/", "/habitat/glossary/", "/habitat/diagrams/"]
gh_repo = "habitat"

[cascade]
  product = ["habitat"]

[menu]
  [menu.habitat]
    title = "About Chef Habitat"
    identifier = "habitat/About Chef Habitat"
    parent = "habitat"
    weight = 5
+++

Chef Habitat offers the ability to build and deploy packages and services across multiple infrastructure environments (bare metal, VM, containers, and PaaS).  Along with this package distribution, Habitat also deploys all necessary dependencies that your package or service needs to run, so that there are no dependencies on system libraries or utilities.

Habitat simplifies the maintenance and improvement of packages by allowing for the ability to quickly rebuild a release when any of the dependencies are updated.  Maintaining current dependencies allows the developer to ensure that the packages they are reliant upon do not have any bugs or vulnerabilities where they need to wait for a system operator to apply a fix.

Releases are easily distributed via communication between the [Habitat Supervisor]({{< relref "sup" >}}) and the [Habitat Builder]({{< relref "builder_overview" >}}). New releases are posted to Builder, and when the Supervisor checks in and sees the new release, it pulls the updated package down and deploys it onto the system.

## Basics of a Habitat Package

Habitat can distribute releases of packages that provide either binary distrobutions (i.e. OpenJDK), or services initialized and run by the Supervisor (i.e. NodeJS, or PostgreSQL).

Packages are defined by the use of a [Plan script]({{< relref "plan_writing" >}}).  This is written in shell or powershell syntax depending on what platform the package is intended to run on.  Plans layout all metadata about a package (name, origin, version, etc...), the source of the code making up the package, information about dependencies needed during both run time and build time.  The plan file also defines any exposed service ports so that other services can communicate with your package (i.e. a Tomcat server being able to talk to a Database server)

For [Services]({{< relref "about_services" >}}), packages contain [Hook]({{< relref "application_lifecycle_hooks" >}}) scripts which tell the Supervisor how to install, initialize, run, stop, and monitor the service as needed.  Along with the hooks, a package for a service contains any necessary config file templates, which use TOML input files to help define how a service is configured once deployed.

Habitat packages are built within the [Habitat Studio]({{< relref "studio" >}}), or using the `hab pkg build` CLI command.  This process compiles or manipulates the source to be destributed, prepares it for use, and then packages it along with all necessary dependencies into a Hart file. The Hart file is a custom compression with extra package metadata which can be used by builder and is derived from data in the Plan.  The Hart file is the basic build artifact of a package and is what is transmitted to and from the Builder as packages are deployed and distributed.

## Distributing a Package

Habitat packages are deployed to the Habitat Builder.  In order to logically organize how packages are referenced, packages are built as part of an Origin.  The Origen is used as a logical organizer, usually used to group inter-related packages together, or to identify which organization or individual maintains or distributes a package.  The Origin of a package is defined in the Plan file.

When a Habitat package is built, it is defined by its version and release.  A package might have many releases of a single version, for example using the same version of Tomcat for a service, but updating the package whenever a new version of Java is released.  The version of such a package would not be updated, but when the package is built for distribution, the datetime that the build happens is used as the Release identifier.  This allows for fine-grained intra-package referencing based on how explicitly you declare the Ident of your package dependencies.  The full Ident of the package is defined as `<ORIGIN>/<PKG_NAME>/<VERSION>/<RELEASE>`.

Once a package has been pushed to the Builder, it can be made available to be pulled down by Habitat clients and supervisors by promoting the release to a Channel.  By default, all uploaded packages are part of an Unstable channel, and the Builder UI provides the ability to promote it to a Stable channel. However, Channel names can be anything that you want.  The Channel is simply a tag that is used by clients and Supervisors to determine when it is appropriate to download a new package release.  For instance, a Supervisor listening to the _Prod_ channel will only pull packages down when builder is told to promote a relase to that specific _Prod_ channel.

## Habitat as a Client and a Supervisor

Installing Habitat provides the [hab cli]({{< relref "habitat_cli" >}}), and the ability run the [Habitat Supervisor]({{< relref "sup" >}}) which is used initialize and launch services on a system from Habitat packages.  Right now Supervisor only runs on Linux and Windows OS distributions, but the `hab cli` is also available to run on MacOS where it can be used to build packages.  Configuring the Supervisor, or using the `hab cli` allows the user to manage packages and services on their system, including any necessary updates that are deployed to the Builder.

## Installing Habitat

The Chef Habitat CLI can be [installed]({{< relref "install_habitat" >}}) on Linux, Mac, and Windows.
