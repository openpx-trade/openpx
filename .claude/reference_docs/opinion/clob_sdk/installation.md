# Installation

This guide will help you install the Opinion CLOB SDK and its dependencies.

<https://pypi.org/project/opinion-clob-sdk/>

### Requirements

#### Python Version

* **Python 3.9.10 or higher**&#x20;

Check your Python version:

```bash
python --version  # or python3 --version
```

#### System Requirements

* **Operating Systems**: Linux, macOS, Windows
* **Network**: Internet connection for API access and blockchain RPC
* **Optional**: Git (for development installation)

### Installation Methods

#### Install from PyPI (Recommended)

The simplest way to install the Opinion CLOB SDK is via pip:

```bash
pip install opinion_clob_sdk
```

This will install the latest stable version and all required dependencies.

### Dependencies

The SDK automatically installs the following dependencies:

### Verify Installation

After installation, verify it works:

```python
import opinion_clob_sdk

# Check version
print(opinion_clob_sdk.__version__)  # Should print: 0.1.0 or higher

# Import main classes
from opinion_clob_sdk import Client
from opinion_clob_sdk.model import TopicType, TopicStatus

print("✓ Opinion CLOB SDK installed successfully!")
```

Or run from command line:

```bash
python -c "import opinion_clob_sdk; print('✓ Installed:', opinion_clob_sdk.__version__)"
```

### Virtual Environment (Recommended)

It's best practice to use a virtual environment:

#### Using venv (Built-in)

```bash
# Create virtual environment
python3 -m venv venv

# Activate it
source venv/bin/activate  # macOS/Linux
# or
venv\Scripts\activate     # Windows

# Install SDK
pip install opinion_clob_sdk

# When done, deactivate
deactivate
```

#### Using conda

```bash
# Create environment
conda create -n opinion python=3.11

# Activate it
conda activate opinion

# Install SDK
pip install opinion_clob_sdk

# When done, deactivate
conda deactivate
```

### Upgrading

To upgrade to the latest version:

```bash
pip install --upgrade opinion_clob_sdk
```

To upgrade all dependencies as well:

```bash
pip install --upgrade --force-reinstall opinion_clob_sdk
```

### Uninstalling

To remove the SDK:

```bash
pip uninstall opinion_clob_sdk
```

### Next Steps

Once installed, proceed to:

1. [Quick Start Guide - Build your first application](https://docs.opinion.trade/developer-guide/opinion-clob-sdk/getting-started/quick-start)
2. [Configuration - Set up API keys and credentials](https://docs.opinion.trade/developer-guide/opinion-clob-sdk/getting-started/configuration)
3. [API Reference - Explore available methods](https://docs.opinion.trade/developer-guide/opinion-clob-sdk/api-references)

***

**Having issues?** Check the Troubleshooting Guide or FAQ.
