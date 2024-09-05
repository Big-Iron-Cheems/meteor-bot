package silly

import (
	"encoding/json"
	"github.com/bwmarrin/discordgo"
	"meteor-bot/internal/app/common"
	"net/http"
)

type CatCommand struct {
	common.BaseCommand
}

func NewCatCommand() *CatCommand {
	return &CatCommand{
		BaseCommand: *common.NewCommandBuilder().
			SetName("cat").
			SetDescription("gato").
			Build(),
	}
}

func (c *CatCommand) Build() *discordgo.ApplicationCommand {
	return &discordgo.ApplicationCommand{
		Name:        c.Name(),
		Description: c.Description(),
	}
}

func (c *CatCommand) Handler() common.CommandHandler {
	return func(s *discordgo.Session, i *discordgo.InteractionCreate) {
		// Fetch the cat image
		resp, err := http.Get("https://some-random-api.com/img/cat")
		if err != nil {
			c.HandleInteractionRespond(s, i, &discordgo.InteractionResponse{
				Type: discordgo.InteractionResponseChannelMessageWithSource,
				Data: &discordgo.InteractionResponseData{
					Content: "Failed to fetch cat image",
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
		url, ok := jsonResponse["link"].(string)
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
