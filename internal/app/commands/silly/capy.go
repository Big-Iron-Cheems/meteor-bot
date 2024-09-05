package silly

import (
	"encoding/json"
	"github.com/bwmarrin/discordgo"
	"meteor-bot/internal/app/common"
	"net/http"
)

type CapyCommand struct {
	common.BaseCommand
}

func NewCapyCommand() *CapyCommand {
	return &CapyCommand{
		BaseCommand: *common.NewCommandBuilder().
			SetName("capy").
			SetDescription("pulls up").
			Build(),
	}
}

func (c *CapyCommand) Build() *discordgo.ApplicationCommand {
	return &discordgo.ApplicationCommand{
		Name:        c.Name(),
		Description: c.Description(),
	}
}

func (c *CapyCommand) Handler() common.CommandHandler {
	return func(s *discordgo.Session, i *discordgo.InteractionCreate) {
		// Fetch the capybara image
		resp, err := http.Get("https://api.capy.lol/v1/capybara?json=true")
		if err != nil {
			c.HandleInteractionRespond(s, i, &discordgo.InteractionResponse{
				Type: discordgo.InteractionResponseChannelMessageWithSource,
				Data: &discordgo.InteractionResponseData{
					Content: "Failed to fetch capybara image",
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
					Content: "Failed to decode the response",
					Flags:   discordgo.MessageFlagsEphemeral,
				},
			})
			return
		}

		// Extract the image URL
		data, ok := jsonResponse["data"].(map[string]any)
		if !ok {
			c.HandleInteractionRespond(s, i, &discordgo.InteractionResponse{
				Type: discordgo.InteractionResponseChannelMessageWithSource,
				Data: &discordgo.InteractionResponseData{
					Content: "Failed to parse the response",
					Flags:   discordgo.MessageFlagsEphemeral,
				},
			})
			return
		}
		url, ok := data["url"].(string)
		if !ok {
			c.HandleInteractionRespond(s, i, &discordgo.InteractionResponse{
				Type: discordgo.InteractionResponseChannelMessageWithSource,
				Data: &discordgo.InteractionResponseData{
					Content: "Failed to parse the response",
					Flags:   discordgo.MessageFlagsEphemeral,
				},
			})
			return
		}

		// Respond to the interaction
		c.HandleInteractionRespond(s, i, &discordgo.InteractionResponse{
			Type: discordgo.InteractionResponseChannelMessageWithSource,
			Data: &discordgo.InteractionResponseData{
				Content: url,
			},
		})
	}
}
