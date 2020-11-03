package hsl;

import hsl.HslDataMappings.HslData;
import hsl.HslDataMappings.HslJSONSerde;
import hsl.VehicleSpeedAnalyses.VehicleSpeedStats;
import hsl.VehicleSpeedAnalyses.VehicleSpeedStatsSerde;

import org.apache.kafka.common.serialization.*;
import org.apache.kafka.streams.*;
import org.apache.kafka.streams.kstream.*;
import org.apache.log4j.Logger;

import java.util.Properties;
import java.util.concurrent.CountDownLatch;

public class VehicleSpeedAnalysisStream {

    public static void main(String[] args) throws Exception {
        Properties props = new Properties();
        // Application identifier and kafka broker
        props.put(StreamsConfig.APPLICATION_ID_CONFIG, "hsl-vehicle-speeds");
        props.put(StreamsConfig.BOOTSTRAP_SERVERS_CONFIG, "localhost:9092");
        // Send updates after every record
        props.put(StreamsConfig.CACHE_MAX_BYTES_BUFFERING_CONFIG, 0);

        final StreamsBuilder builder = new StreamsBuilder();

        final KStream<String, HslData> hslData = builder.stream(
                "hsl",
                Consumed.with(Serdes.String(), new HslJSONSerde()));

        final KGroupedStream<String, Double> speeds = hslData
                .map((key, value) -> new KeyValue<>(value.getKey(), value.spd))
                .groupByKey(Grouped.with(Serdes.String(), Serdes.Double()));


       KTable<String, VehicleSpeedStats> stats =
                speeds.aggregate(
                        VehicleSpeedStats::new,
                (key, value, aggregate) -> {
                            aggregate.incCount();
                            aggregate.incSpeedSum(value);
                            return aggregate;
                }, Materialized.with(Serdes.String(), new VehicleSpeedStatsSerde()));

        final KTable<String, Double> avgSpeed =
                stats.mapValues(
                        VehicleSpeedStats::calcAvg,
                        Materialized.with(Serdes.String(), Serdes.Double())
                );

        avgSpeed.toStream()
                .to("hsl-speed-stats", Produced.with(Serdes.String(), Serdes.Double()));

        final Topology topology = builder.build();

        final KafkaStreams streams = new KafkaStreams(topology, props);
        final CountDownLatch latch = new CountDownLatch(1);

        // attach shutdown handler to catch control-c
        // From quickstart tutorial https://kafka.apache.org/26/documentation/streams/tutorial
        Runtime.getRuntime().addShutdownHook(new Thread("hsl-streams-shutdown-hook") {
            @Override
            public void run() {
                streams.close();
                latch.countDown();
            }
        });

        try {
            streams.start();
            latch.await();
        } catch (Throwable e) {
            System.exit(1);
        }
        System.exit(0);
    }
}
