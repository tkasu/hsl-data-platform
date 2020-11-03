package hsl;

import com.google.gson.Gson;
import org.apache.kafka.common.serialization.Deserializer;
import org.apache.kafka.common.serialization.Serde;
import org.apache.kafka.common.serialization.Serializer;
import org.apache.log4j.Logger;

import java.nio.charset.StandardCharsets;
import java.util.Map;

public class VehicleSpeedAnalyses {


    public static class VehicleSpeedStatsSerde implements Serializer<VehicleSpeedStats>, Deserializer<VehicleSpeedStats>, Serde<VehicleSpeedStats> {
        private static Gson gson;
        final Logger logger = Logger.getLogger(VehicleSpeedAnalyses.class.getName());

        public VehicleSpeedStatsSerde() {
            gson = new Gson();
        }

        @Override
        public VehicleSpeedStats deserialize(String topic, byte[] data) {
            if (data == null) {
                return null;
            }

            try {
                String jsonString = new String(data, StandardCharsets.UTF_8);
                return gson.fromJson(jsonString, VehicleSpeedStats.class);
            } catch (final Exception e) {
                logger.warn("Could not deserialize data to VehicleSpeedStats: " + data);
                return null;
            }
        }

        @Override
        public Serializer<VehicleSpeedStats> serializer () { return this; }

        @Override
        public Deserializer<VehicleSpeedStats> deserializer () { return this; }

        @Override
        public void configure (Map<String, ?> configs, boolean isKey) {}

        @Override
        public byte[] serialize (String topic, VehicleSpeedStats data){
            if (data == null) {
                return null;
            }

            try {
                return gson.toJson(data).getBytes(StandardCharsets.UTF_8);
            } catch (final Exception e) {
                logger.warn("Could not serialize class VehicleSpeedStats: " + data.toString());
                return null;
            }
        }

        @Override
        public void close () { }
    }

    public static class VehicleSpeedStats {
        public Long count;
        public Double sumSpeed;

        public VehicleSpeedStats() {
            this.count = 0L;
            this.sumSpeed = 0.0;
        }

        public VehicleSpeedStats(Long initCount, Double initSum) {
            this.count = initCount;
            this.sumSpeed = initSum;
        }

        public Double calcAvg() {
            return this.sumSpeed / this.count;
        }

        public Long getCount() {
            return this.count;
        }

        public Double getSumSpeed() {
            return this.sumSpeed;
        }

        public void incCount() {
            this.count++;
        }

        public void incSpeedSum(Double speed) {
            this.sumSpeed += speed;
        }

        public void update(HslDataMappings.HslData data) {
            this.incCount();
            this.incSpeedSum(data.spd);
        }

    }

}



