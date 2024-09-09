package common

import (
	"github.com/rs/zerolog"
	"github.com/rs/zerolog/log"
	"meteor-bot/internal/app/config"
	"os"
	"path"
	"time"
)

// logFile is the file to write logs to
var logFile *os.File

func InitLogger() {
	zerolog.TimeFieldFormat = zerolog.TimeFormatUnix

	if config.GlobalConfig.EnableLogFile {
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
	} else {
		log.Logger = log.Output(zerolog.ConsoleWriter{
			Out:        os.Stdout,
			TimeFormat: time.TimeOnly,
		})
	}

	log.Info().Msg("Logger initialized")
}

func CloseLogger() {
	if logFile != nil {
		if err := logFile.Close(); err != nil {
			log.Error().Err(err).Msg("Failed to close log file")
		}
	}
}
