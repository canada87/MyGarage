-- Home Assistant sensor (entity_id) that reports the car's odometer, e.g. sensor.yaris_cross_odometer.
ALTER TABLE vehicles ADD COLUMN ha_entity TEXT;
