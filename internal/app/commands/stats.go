package commands

import (
	"encoding/json"
	"fmt"
	"github.com/bwmarrin/discordgo"
	"meteor-bot/internal/app/common"
	"meteor-bot/internal/app/config"
	"net/http"
	"regexp"
	"time"
)

type StatsCommand struct {
	common.BaseCommand
}

func NewStatsCommand() *StatsCommand {
	return &StatsCommand{
		BaseCommand: *common.NewCommandBuilder().
			SetName("stats").
			SetDescription("Shows various stats about Meteor").
			Build(),
	}
}

func (c *StatsCommand) Build() *discordgo.ApplicationCommand {
	return &discordgo.ApplicationCommand{
		Name:        c.Name(),
		Description: c.Description(),
		Options: []*discordgo.ApplicationCommandOption{
			{
				Name:        "date",
				Description: "The date to fetch the stats for",
				Type:        discordgo.ApplicationCommandOptionString,
				Required:    false,
			},
		},
	}
}

func (c *StatsCommand) Handler() common.CommandHandler {
	return func(s *discordgo.Session, i *discordgo.InteractionCreate) {
		// Fetch optional arguments
		date := time.Now().Format("02-01-2006")
		for _, opt := range i.ApplicationCommandData().Options {
			if opt.Name == "date" {
				dateValue := opt.StringValue()
				if dateValue != "" {
					if !regexp.MustCompile(`\d{2}-\d{2}-\d{4}`).MatchString(dateValue) {
						c.HandleInteractionRespond(s, i, &discordgo.InteractionResponse{
							Type: discordgo.InteractionResponseChannelMessageWithSource,
							Data: &discordgo.InteractionResponseData{
								Content: "Invalid date format. Please use DD-MM-YYYY.",
								Flags:   discordgo.MessageFlagsEphemeral,
							},
						})
						return
					}
					date = dateValue
				}
				break
			}
		}

		// Fetch the stats for the given date
		// TODO: make a util function for the request
		resp, err := http.Get(fmt.Sprintf("%s/stats?date=%s", config.GlobalConfig.ApiBase, date))
		if err != nil || resp.StatusCode != http.StatusOK {
			c.HandleInteractionRespond(s, i, &discordgo.InteractionResponse{
				Type: discordgo.InteractionResponseChannelMessageWithSource,
				Data: &discordgo.InteractionResponseData{
					Content: "Failed to fetch stats for this date.",
					Flags:   discordgo.MessageFlagsEphemeral,
				},
			})
			return
		}
		defer resp.Body.Close()

		// Decode the response
		var jsonResponse map[string]any
		if err = json.NewDecoder(resp.Body).Decode(&jsonResponse); err != nil {
			c.HandleInteractionRespond(s, i, &discordgo.InteractionResponse{
				Type: discordgo.InteractionResponseChannelMessageWithSource,
				Data: &discordgo.InteractionResponseData{
					Content: "Failed to decode the response.",
					Flags:   discordgo.MessageFlagsEphemeral,
				},
			})
			return
		}

		// Check for errors in response
		if _, ok := jsonResponse["error"]; ok {
			c.HandleInteractionRespond(s, i, &discordgo.InteractionResponse{
				Type: discordgo.InteractionResponseChannelMessageWithSource,
				Data: &discordgo.InteractionResponseData{
					Content: "Failed to fetch stats for this date.",
					Flags:   discordgo.MessageFlagsEphemeral,
				},
			})
			return
		}

		// Parse the response
		respDate := jsonResponse["date"].(string)
		joins := int(jsonResponse["joins"].(float64))
		leaves := int(jsonResponse["leaves"].(float64))
		gained := joins - leaves
		downloads := int(jsonResponse["downloads"].(float64))

		// Respond to the interaction
		content := fmt.Sprintf("**Date**: %s\n**Joins**: %d\n**Leaves**: %d\n**Gained**: %d\n**Downloads**: %d", respDate, joins, leaves, gained, downloads)
		c.HandleInteractionRespond(s, i, &discordgo.InteractionResponse{
			Type: discordgo.InteractionResponseChannelMessageWithSource,
			Data: &discordgo.InteractionResponseData{
				Embeds: []*discordgo.MessageEmbed{
					{
						Title:       "Meteor Stats",
						Description: content,
						Color:       common.EmbedColor,
					},
				},
			},
		})
	}
}
