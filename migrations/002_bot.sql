-- List of channels to send notifications to.
CREATE TABLE discord_channels (
    guild_id INTEGER,
    channel_id INTEGER
);

-- Always include the bottest channel
INSERT INTO discord_channels(guild_id, channel_id) VALUES (536182478780760065, 536192255900385280);