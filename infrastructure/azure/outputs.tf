output "creds" {
  value = {
    eventhub_conn_str = azurerm_eventhub_namespace_authorization_rule.hsl_eventhub_auth.primary_connection_string
  }
}
