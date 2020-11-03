package hsl;

import com.fasterxml.jackson.annotation.JsonAlias;
import com.fasterxml.jackson.databind.ObjectMapper;
import org.apache.kafka.common.errors.SerializationException;
import org.apache.kafka.common.serialization.Deserializer;
import org.apache.kafka.common.serialization.Serde;
import org.apache.kafka.common.serialization.Serializer;

import java.io.IOException;
import java.util.Map;

public class HslDataMappings {

    public static class HslJSONSerde implements Serializer<HslData>, Deserializer<HslData>, Serde<HslData> {

        private static final ObjectMapper OBJECT_MAPPER = new ObjectMapper();

        @Override
        public HslData deserialize(final String topic, final byte[] data) {
            if (data == null) {
                return null;
            }

            try {
                return OBJECT_MAPPER.readValue(data, HslData.class);
            } catch (final IOException e) {
                throw new SerializationException(e);
            }
        }

        @Override
        public Serializer<HslData> serializer() {
            return this;
        }

        @Override
        public Deserializer<HslData> deserializer() {
            return this;
        }

        @Override
        public void configure(Map<String, ?> configs, boolean isKey) {}

        @Override
        public byte[] serialize(String topic, HslData data) {
            if (data == null) {
                return null;
            }

            try {
                return OBJECT_MAPPER.writeValueAsBytes(data);
            } catch (final Exception e) {
                throw new SerializationException("Error serializing JSON message", e);
            }
        }

        @Override
        public void close() {}
    }

    public static class HslData {
        /*
        Example data:
        {"acc":0.04,"
        desi":"543",
        "dir":"1",
        "dl":-43,
        "drst":1,
        "hdg":325,
        "jrn":137,
        "lat":60.210026,
        "line":260,
        "loc":"GPS",
        "long":24.762691,
        "occu":0,
        "oday":"2020-11-01",
        "odo":4622,
        "oper":22,
        "route":"2543",
        "spd":0.09,
        "start":"15:26",
        "stop":2131264,
        "tsi":1604237856,
        "tst":"2020-11-01T13:37:36.321Z",
        "veh":1217

        TODO: Fix e.g. date datatypes
         */
        public Integer acc;
        public String desi;
        public String dir;
        public Integer dl;
        public Integer drst;
        public Integer hdg;
        public Integer jrn;
        @JsonAlias({ "lat" })
        public Double lattitude;
        public Integer line;
        public String loc;
        @JsonAlias({ "long" })
        public Double longitude;
        public Integer occu;
        public String oday;
        public Integer odo;
        public Integer oper;
        public String route;
        public Double spd;
        public String start;
        public Long stop;
        public Long tsi;
        public String tst;
        public Integer veh;

        public String getKey() {
            return this.desi + "-" + this.dir + "-" + this.veh.toString();
        }
    }
}
