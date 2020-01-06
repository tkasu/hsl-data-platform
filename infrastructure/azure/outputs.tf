output "creds" {
  value = {
    eventhub_conn_str = azurerm_eventhub_namespace_authorization_rule.hsl_eventhub_auth.primary_connection_string
    storage_account_key = azurerm_storage_account.storage_account.primary_access_key
  }
}

output "names" {
  value = {
    storage_account_name = azurerm_storage_account.storage_account.name
    storage_container_name = azurerm_storage_container.blob_storage.name
  }
}
