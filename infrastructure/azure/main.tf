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

resource "azurerm_eventhub_namespace" "eventhub_namespace" {
  name                = "${var.namespace}-hubs"
  location            = azurerm_resource_group.default.location
  resource_group_name = azurerm_resource_group.default.name
  sku                 = "Standard"
  capacity            = 1
  auto_inflate_enabled = true
  maximum_throughput_units = 5

  tags = {
    User = "ADE"
  }
}

resource "azurerm_eventhub" "eventhub" {
  name                = "weatherstats"
  namespace_name      = azurerm_eventhub_namespace.eventhub_namespace.name
  resource_group_name = azurerm_resource_group.default.name
  partition_count     = 2
  message_retention   = 1
}
