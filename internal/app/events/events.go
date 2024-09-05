package events

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"github.com/bwmarrin/discordgo"
	"github.com/rs/zerolog/log"
	"math"
	"meteor-bot/internal/app/config"
	"net/http"
	"strconv"
	"strings"
	"sync/atomic"
	"time"
)

const (
	// infoChannel update period
	updatePeriod = 6 * time.Minute
)

var (
	// Can and must be set only in discordgo.Ready handlers
	guild *discordgo.Guild

	// hello event
	greetings = []string{"hi", "hello", "howdy", "bonjour", "ciao", "hej", "hola", "yo"}
	copeEmoji *discordgo.Emoji

	// infoChannel event
	suffixes = []string{"k", "m", "b", "t"}
	delay    int64

	// metrics event
	metricsServer *http.Server
)

func Init(s *discordgo.Session) {
	registerEventHandlers(s)
}

// registerEventHandlers registers non-command event handlers to the Discord session
func registerEventHandlers(s *discordgo.Session) {
	var err error
	if config.GlobalConfig.CopeNnId != "" {
		copeEmoji, err = s.GuildEmoji(config.GlobalConfig.GuildId, config.GlobalConfig.CopeNnId)
		if err != nil {
			log.Error().Err(err).Msg("Failed to get emoji")
		}
	} else {
		log.Warn().Msg("CopeNnId is not set, skipping emoji fetch")
	}

	s.AddHandler(helloHandler)
	s.AddHandler(userJoinedHandler)
	s.AddHandler(userLeftHandler)
	s.AddHandler(botStartHandler)
	s.AddHandler(uptimeReadyHandler)
	s.AddHandler(infoChannelHandler)
	s.AddHandler(metricsReadyHandler)
	s.AddHandler(metricsDisconnectHandler)

	log.Info().Msg("Events registered successfully.")
}

// helloHandler handles the event when the bot is mentioned
func helloHandler(s *discordgo.Session, m *discordgo.MessageCreate) {
	// ignore self messages
	if m.Author.ID == s.State.User.ID {
		return
	}

	// Check if the message is from a text channel and the bot is mentioned
	if m.GuildID != config.GlobalConfig.GuildId || !strings.Contains(m.Content, s.State.User.Mention()) {
		return
	}

	// Check if the message contains a greeting
	for _, greeting := range greetings {
		if strings.Contains(strings.ToLower(m.Content), greeting) {
			_, _ = s.ChannelMessageSendReply(m.ChannelID, greeting+" :)", m.Reference())
			return
		}
	}

	if strings.Contains(strings.ToLower(m.Content), "cope") && copeEmoji != nil {
		_ = s.MessageReactionAdd(m.ChannelID, m.ID, copeEmoji.APIName())
	} else {
		_ = s.MessageReactionAdd(m.ChannelID, m.ID, "👋")
	}
}

// userJoinedHandler handles the event when a user joins the server
func userJoinedHandler(_ *discordgo.Session, m *discordgo.GuildMemberAdd) {
	if config.GlobalConfig.BackendToken == "" {
		return
	}

	// POST request to the backend
	req, err := http.NewRequest("POST", config.GlobalConfig.ApiBase+"/discord/userJoined?id="+m.User.ID, nil)
	if err != nil {
		log.Error().Err(err).Msg("Failed to create request")
		return
	}
	req.Header.Set("Authorization", config.GlobalConfig.BackendToken)

	resp, err := http.DefaultClient.Do(req)
	if err != nil {
		log.Error().Err(err).Msg("Failed to send request")
		return
	}
	defer resp.Body.Close()
}

// userLeftHandler handles the event when a user leaves the server
func userLeftHandler(_ *discordgo.Session, m *discordgo.GuildMemberRemove) {
	if config.GlobalConfig.BackendToken == "" {
		return
	}

	// POST request to the backend
	req, err := http.NewRequest("POST", config.GlobalConfig.ApiBase+"/discord/userLeft?id="+m.User.ID, nil)
	if err != nil {
		log.Error().Err(err).Msg("Failed to create request")
		return
	}
	req.Header.Set("Authorization", config.GlobalConfig.BackendToken)

	resp, err := http.DefaultClient.Do(req)
	if err != nil {
		log.Error().Err(err).Msg("Failed to send request")
		return
	}
	defer resp.Body.Close()
}

// botStartHandler sets up the bot status
func botStartHandler(s *discordgo.Session, _ *discordgo.Ready) {
	if err := s.UpdateGameStatus(0, "Meteor Client"); err != nil {
		log.Warn().Err(err).Msg("Failed to set game status")
	}
	log.Info().Msgf("Logged in as: %v#%v", s.State.User.Username, s.State.User.Discriminator)
}

// uptimeReadyHandler sends an uptime request to the configured URL every 60 seconds
func uptimeReadyHandler(s *discordgo.Session, _ *discordgo.Ready) {
	if config.GlobalConfig.UptimeUrl == "" {
		log.Warn().Msg("Uptime URL not set, uptime requests will not be made")
		return
	}

	// Send an uptime request every 60 seconds
	ticker := time.NewTicker(60 * time.Second)
	go func() {
		for {
			select {
			case <-ticker.C:
				url := config.GlobalConfig.UptimeUrl
				if strings.HasSuffix(url, "ping=") {
					url += fmt.Sprintf("%d", s.HeartbeatLatency().Milliseconds())
				}

				resp, err := http.Get(url)
				if err != nil {
					log.Error().Err(err).Msg("Failed to send uptime request")
				} else {
					resp.Body.Close()
				}
			}
		}
	}()
}

// infoChannelHandler updates the member count and download count channels
func infoChannelHandler(s *discordgo.Session, _ *discordgo.Ready) {
	var err error
	guild, err = s.Guild(config.GlobalConfig.GuildId)
	if err != nil || guild == nil {
		log.Warn().Msg("Guild not set, info channels will not be updated")
		return
	}

	if config.GlobalConfig.MemberCountId == "" || config.GlobalConfig.DownloadCountId == "" {
		log.Warn().Msg("Member count or download count channel IDs not set, info channels will not be updated")
		return
	}

	memberCountChannel, err := s.Channel(config.GlobalConfig.MemberCountId)
	if err != nil || memberCountChannel == nil {
		log.Warn().Err(err).Msg("Failed to get member count channel")
		return
	}

	downloadCountChannel, err := s.Channel(config.GlobalConfig.DownloadCountId)
	if err != nil || downloadCountChannel == nil {
		log.Warn().Err(err).Msg("Failed to get download count channel")
		return
	}

	updateChannel(s, downloadCountChannel, func() int64 {
		resp, err := http.Get(config.GlobalConfig.ApiBase + "/stats")
		if err != nil {
			log.Error().Err(err).Msg("Failed to fetch download stats")
			return 0
		}
		defer resp.Body.Close()

		var stats map[string]any
		if err := json.NewDecoder(resp.Body).Decode(&stats); err != nil {
			log.Error().Err(err).Msg("Failed to parse download stats")
			return 0
		}

		return stats["downloads"].(int64)
	})

	updateChannel(s, memberCountChannel, func() int64 {
		return int64(guild.MemberCount)
	})
}

// metricsReadyHandler starts the metrics server
func metricsReadyHandler(s *discordgo.Session, _ *discordgo.Ready) {
	var err error
	guild, err = s.Guild(config.GlobalConfig.GuildId)
	if err != nil || guild == nil {
		log.Warn().Msg("Guild not set, metrics server will not be started")
		return
	}

	http.HandleFunc("/metrics", onRequest)
	metricsServer = &http.Server{Addr: ":9400"}

	go func() {
		if err := metricsServer.ListenAndServe(); err != nil && !errors.Is(err, http.ErrServerClosed) {
			log.Panic().Err(err).Msg("Failed to start metrics server")
		}
	}()

	log.Info().Msg("Providing metrics on :9400/metrics")
}

// metricsDisconnectHandler shuts down the metrics server when the bot disconnects
func metricsDisconnectHandler(_ *discordgo.Session, _ *discordgo.Disconnect) {
	if metricsServer != nil {
		if err := metricsServer.Shutdown(context.Background()); err != nil {
			log.Error().Err(err).Msg("Failed to shutdown metrics server")
		} else {
			log.Info().Msg("Metrics server shutdown gracefully")
		}
	}
}

// updateChannel updates the channel name with the given supplier function
func updateChannel(s *discordgo.Session, channel *discordgo.Channel, supplier func() int64) {
	atomic.AddInt64(&delay, int64(updatePeriod/2))

	go func() {
		ticker := time.NewTicker(updatePeriod)
		defer ticker.Stop()

		for {
			select {
			case <-ticker.C:
				name := channel.Name
				newName := fmt.Sprintf("%s %s", name[:strings.LastIndex(name, ":")+1], formatLong(supplier()))
				_, err := s.ChannelEdit(channel.ID, &discordgo.ChannelEdit{Name: newName})
				if err != nil {
					log.Error().Err(err).Msg("Failed to update channel name")
				}
			}
		}
	}()
}

// formatLong formats a long number into a human-readable string
func formatLong(value int64) string {
	if value < 1000 {
		return strconv.FormatInt(value, 10)
	}

	exponent := int(math.Log10(float64(value)) / 3)
	if exponent > len(suffixes) {
		exponent = len(suffixes)
	}

	base := math.Pow(1000, float64(exponent))
	first := float64(value) / base
	return fmt.Sprintf("%.2f%s", first, suffixes[exponent-1])
}

// onRequest handles the metrics request
func onRequest(w http.ResponseWriter, _ *http.Request) {
	response := fmt.Sprintf(
		`# HELP meteor_discord_users_total Total number of Discord users in our server
# TYPE meteor_discord_users_total gauge
meteor_discord_users_total %d`,
		guild.MemberCount)

	w.Header().Set("Content-Type", "text/plain")
	w.WriteHeader(http.StatusOK)
	_, err := w.Write([]byte(response))
	if err != nil {
		log.Error().Err(err).Msg("Failed to write response")
	}
}
