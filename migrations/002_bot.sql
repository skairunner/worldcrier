-- List of channels to send notifications to.
CREATE TABLE discord_channels (
    guild_id TEXT NOT NULL,
    channel_id TEXT NOT NULL
);

CREATE UNIQUE INDEX idx_discord_channels_channel_id ON discord_channels (channel_id);

-- Always include the bottest channel
INSERT INTO discord_channels(guild_id, channel_id) VALUES ('536182478780760065', '536192255900385280');