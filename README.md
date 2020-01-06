# HSL Data Platform

Project includes Infrastructure as Code template and application to get data vehicle position data from [HSL's High-frequency position API](https://digitransit.fi/en/developers/apis/4-realtime-api/vehicle-positions/) and forward it to Azure for further analytics.

The main data flows are following:

For Batch Analytics:

HSL Realtime Mqtt API -> Rust mini application -> Azure Event Hub -> Azure Strem Analytics -> Azure Blob Storage -> Azure Databricks

For Stream Analytics:

HSL Realtime Mqtt API -> Rust mini application -> Azure Event Hub -> Azure Databricks

For me, this project was a great tool to learn more about Azure's data stack and to analyze real data with Azure Databricks.

## Installation

Requires [Terraform](https://www.terraform.io/) installation.

### Init Azure infrastrucutre

Project uses Terraform to provision and manage infrastrucure. Credentials are not in terraform files, as I'm using [VS Code's Azure Terraform plugin](https://marketplace.visualstudio.com/items?itemName=ms-azuretools.vscode-azureterraform) to deploy infrastrucure directly from Azure Cloud Shell. If you like to use alterantive approach, see [azurem documentation](https://www.terraform.io/docs/providers/azurerm/index.html).

With credentials in place:

```bash
cd ./infrastructure/azure
terraform init
terraform apply
```

Save outputted eventhub_conn_str for latere.

### Vehicle Position Data Forwarder application

[Vehicle Position Data Forwarder Application](applications/vehicle-pos-data-forwarder/) requires either [rust](https://waww.rust-lang.org) or [docker](https://www.docker.com/) installation. See applications/vehicle-pos-data-forwarder/README.md for more details. 

## Usage

### Generate events to azure

Spin up one (or more) [Vehicle Position Data Forwarder Application](applications/vehicle-pos-data-forwarder/) applications.

### Start Azure Stream Analytics

Start ASA job from either Azure Portal or using Azure Cloud Shell.

### Create Azure Databricks Service & Cluster

Create Azure Databricks Cluster with Azure Portal.

### Analyze data with Databricks!

#### Batch analytics example from Blob storage

Mount your blob storage with [these instructions](https://docs.microsoft.com/en-gb/azure/databricks/data/data-sources/azure/azure-storage).

TODO Automatically set up Azure Key Vault with Terraform.

After that, access the mount with Spark:

```scala
// blob storage mounted to /mnt/hsl_test/
val df = spark.read.format("avro").load("/mnt/hsl_test/avro/*/*/")

display(df)
``` 

## License
[MIT](https://choosealicense.com/licenses/mit/)
