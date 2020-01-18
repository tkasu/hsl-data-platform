// Databricks notebook source
spark

// COMMAND ----------

dbutils.fs.mount(
  source = "wasbs://hsl-dev-westeur-blob-storage@FIXME",
  mountPoint = "/mnt/hsl_test",
  extraConfigs = Map("fs.azure.account.key.FIXME" -> dbutils.secrets.get(scope = "hsl_test", key = "hslblob")))

// COMMAND ----------

// blob storage mounted to /mnt/hsl_test/
val df = spark.read.format("avro").load("/mnt/hsl_test/avro/*/*/")

// COMMAND ----------

display(df)

// COMMAND ----------


