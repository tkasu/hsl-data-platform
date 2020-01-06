resource "azurerm_resource_group" "default" {
  name     = var.namespace
  location = var.region
}

resource "random_string" "rand" {
  length  = 24
  special = false
  upper   = false
}

resource "azurerm_storage_account" "storage_account" {
  name                     = random_string.rand.result
  resource_group_name      = azurerm_resource_group.default.name
  location                 = azurerm_resource_group.default.location
  account_tier             = "Standard"
  account_replication_type = "LRS"
}


resource "azurerm_storage_container" "blob_storage" {
  name                  = "${var.namespace}-blob-storage"
  storage_account_name  = azurerm_storage_account.storage_account.name
  container_access_type = "private"
}

resource "azurerm_eventhub_namespace" "eventhub_namespace" {
  name                = "${var.namespace}-hubs"
  location            = azurerm_resource_group.default.location
  resource_group_name = azurerm_resource_group.default.name
  sku                 = "Standard"
  capacity            = 1
  auto_inflate_enabled = true
  maximum_throughput_units = 5

  tags = {
    User = "HSL"
  }
}

resource "azurerm_eventhub_namespace_authorization_rule" "hsl_eventhub_auth" {
  name                = "${var.namespace}-hubs-auth-hsl-sender"
  namespace_name      = azurerm_eventhub_namespace.eventhub_namespace.name
  resource_group_name = azurerm_resource_group.default.name

  listen = true
  send   = true
  manage = false
}

resource "azurerm_eventhub" "eventhub" {
  name                = "vehiclepos"
  namespace_name      = azurerm_eventhub_namespace.eventhub_namespace.name
  resource_group_name = azurerm_resource_group.default.name
  partition_count     = 2
  message_retention   = 1
}

resource "azurerm_eventhub_consumer_group" "eventhub_consumergroup" {
  name                = "vehiclepos-consumergroup"
  namespace_name      = azurerm_eventhub_namespace.eventhub_namespace.name
  eventhub_name       = azurerm_eventhub.eventhub.name
  resource_group_name = azurerm_resource_group.default.name
}

resource "azurerm_stream_analytics_job" "sas_job" {
  name                                     = "${var.namespace}-sas-hsl-toblob"
  resource_group_name                      = azurerm_resource_group.default.name
  location                                 = azurerm_resource_group.default.location
  compatibility_level                      = "1.1"
  data_locale                              = "en-GB"
  events_late_arrival_max_delay_in_seconds = 60
  events_out_of_order_max_delay_in_seconds = 50
  events_out_of_order_policy               = "Adjust"
  output_error_policy                      = "Drop"
  streaming_units                          = 3

  tags = {
    User = "HSL"
  }

  transformation_query = <<QUERY
    SELECT *
    INTO [hsl-blob-output]
    FROM [hsl-hub-input]
QUERY
}

resource "azurerm_stream_analytics_stream_input_eventhub" "sas_job_input" {
  name                         = "hsl-hub-input"
  stream_analytics_job_name    = azurerm_stream_analytics_job.sas_job.name
  resource_group_name          = azurerm_stream_analytics_job.sas_job.resource_group_name
  eventhub_consumer_group_name = azurerm_eventhub_consumer_group.eventhub_consumergroup.name
  eventhub_name                = azurerm_eventhub.eventhub.name
  servicebus_namespace         = azurerm_eventhub_namespace.eventhub_namespace.name
  shared_access_policy_key     = azurerm_eventhub_namespace_authorization_rule.hsl_eventhub_auth.primary_key
  shared_access_policy_name    = azurerm_eventhub_namespace_authorization_rule.hsl_eventhub_auth.name

  serialization {
    type     = "Json"
    encoding = "UTF8"
  }
}

resource "azurerm_stream_analytics_output_blob" "sas_job_output" {
  name                      = "hsl-blob-output"
  stream_analytics_job_name = azurerm_stream_analytics_job.sas_job.name
  resource_group_name       = azurerm_stream_analytics_job.sas_job.resource_group_name
  storage_account_name      = azurerm_storage_account.storage_account.name
  storage_account_key       = azurerm_storage_account.storage_account.primary_access_key
  storage_container_name    = azurerm_storage_container.blob_storage.name
  path_pattern              = "avro/{date}/{time}"
  date_format               = "yyyy-MM-dd"
  time_format               = "HH"

  serialization {
    type            = "Avro"
  }
}