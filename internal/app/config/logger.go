package config

import (
	"github.com/rs/zerolog"
	"github.com/rs/zerolog/log"
	"os"
	"path"
)

// logFile is the file to write logs to
var logFile *os.File

func init() {
	zerolog.TimeFieldFormat = zerolog.TimeFormatUnix
	var err error
	logFile, err = os.OpenFile(
		path.Join("logs", "meteor-bot.log"),
		os.O_APPEND|os.O_CREATE|os.O_WRONLY,
		0664,
	)
	if err != nil {
		log.Panic().Err(err).Msg("Failed to open log file")
	}

	consoleWriter := zerolog.ConsoleWriter{Out: os.Stdout}
	multi := zerolog.MultiLevelWriter(consoleWriter, logFile)

	// Set the default logger to write to both console and file
	log.Logger = zerolog.New(multi).With().Timestamp().Logger()
	log.Debug().Msg("Logger initialized")
}

// CloseLogFile closes the log file
func CloseLogFile() {
	if logFile != nil {
		if err := logFile.Close(); err != nil {
			log.Error().Err(err).Msg("Failed to close log file")
		}
	}
}
