create table notify (
	channel UNSIGNED BIG INT,
	-- 00 my        00 all
	-- 01 all       01 movies
	-- 10 undefined   10 shows
	-- 11 undefined 11 undefined
	type INT(4) DEFAULT 0,
	-- discord_id for my
	-- null for all
	data UNSIGNED BIG INT,
	CONSTRAINT PK_notify PRIMARY KEY (channel, type, data)
);

