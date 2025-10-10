## Colony

Colony is both a small framework revolving around Singularity Containers and a Graphical User Interface that handles most manual steps around Container Usage. Its main focus lies on simplicity for the user in order to foster reproducibility and accelerate the adoption of new tools.
Containerization allows you to take arbitrary software stacks and package them into one entity - in case of Singularity that entity is a regular file. This makes entire analysis workflows portable and installation free. Everything you need is Singularity and the Container file.
Actually using these Containers, however, often requires much more. You need to manage data (including the Containers), learn how to use the software and then find the correct configuration for your use case. That is where the Colony Launcher GUI helps you - it guides your decision making while organizing, learning and applying your workflows.

The core functionality of Colony workflows requires no software other than Singularity itself.
Enforcing a two-step procedure, usage of Colony necessarily documents itself. First, the user creates a usage manifest that may come from a container-internal GUI. Then the user is able use both the container and the manifest file to run the workflow.
Not only does this reduce the need for manual documentation, it unifies the usage of software, and also allows the Colony Launcher GUI to reason about software usage and take automation to the next level.

If you are interested check out Colony's webpage at https://clipc-jpg.github.io/ColonyWebsite/.  
If you meet all requirements you may also build the project from source using cargo.

## Why Colony?

The project emerged from the need for more reproducible research.
Reproducibility is a benefit to researchers and what makes research more reproducible does not have clear boundaries.
In the context of genomic research, scientists are confronted with rapidly increasing amounts of data. They take raw or basecalled measurements from DNA/RNA sequencers from various vendors and use the vendors' proprietary software to distill them to smaller datasets that they can further analyze interactively.
They are confronted with two major problems:
- The analysis as a whole requires large computing resources that cannot fit onto an ordinary laptop. They have to own or rent a large, expensive workstation and learn the framework that gives them access to that computer.
- Bespoken proprietary software is often coupled to the vendor's own hardware in order to simplify its use and/or contains a steep learning curve. This effectively leads to vendor lock-in.

Many open source solutions exist, but they tend to have other limitations, such as 

- Limited user friendliness, e.g. correct usage consists of many individual steps
- Risk of nontrivial misuse of the software
- Compatibility issues: They require installation of specific software versions on specific operating systems.
- Instability: The Software may work at a specific point in time on a large number of machines, but changes over time create incompatibilities with extensions/workflows written for older versions of the software or the operating system. Identifying and fixing these problems is a major loss of the reasearcher's time.
- Steep learning curve: There is very many software solutions using many different algorithms and each one of them has to be understood not only in how they work, but also in how they need to be used. This is a major hindrance to adoption of new approaches.

Colony tries to prevent these issues by removing as many moving parts as possible - the analysis may be exported into a completely portable repository. We aim to build upon the achieved equivariance of usage and extend automation to more tasks in order to accelerate Research Data Management.


# Development

This project is a workspace dedicated to a desktop application written for Windows. It uses WSL under the hood.
The underlying technology is Dioxus, which is similar to React websites, and uses CSS for styling.
For Development, you will need to install rust, and with it comes its package manager cargo.

Currently, compilation is done with raw cargo commands.
The actual desktop application is located in the subdirectory "userfrontendexecutable" and the project inside is called "ColonyLauncher".
You can run the program with the command
```bash
cargo run -p ColonyLauncher --release
```
However, you need to manually copy assets to the correct place, in order to be rendered in the GUI (currently, a logo and the background image, which you can find under inno_setup/assets/).

To alleviate the issue, the buildtask-pattern needs to be completed, like the one used in the wf-transcriptomes project.


In order to build the installer, you need to install InnoSetup and create a ColonyWSL.tar file and move it into inno_setup/assets.
You may reuse a ColonyWSL.tar from a previous release. Otherwise:
- Create a fresh Ubuntu WSL distribution.
- Export it to a Ubuntu.tar file.
- Import the Ubuntu.tar file as ColonyWSL
- Install necessary dependencies. Currently that is simply singularity and in future releases you might need curl amongst others.
- Export ColonyWSL to inno_setup/assets/ColonyWSL.tar

Now run InnoSetup on the setup_userspace.ini:
```bash
cd inno_setup/assets/setup_userspace.ini
ISCC setup_userspace.ini
```

This builds the installer at inno_setup/inno_setup_output


































































