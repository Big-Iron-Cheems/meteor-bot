package silly

import (
	"encoding/json"
	"github.com/bwmarrin/discordgo"
	"meteor-bot/internal/app/common"
	"net/http"
)

type DogCommand struct {
	common.BaseCommand
}

func NewDogCommand() *DogCommand {
	return &DogCommand{
		BaseCommand: *common.NewCommandBuilder().
			SetName("dog").
			SetDescription("dawg").
			Build(),
	}
}

func (c *DogCommand) Build() *discordgo.ApplicationCommand {
	return &discordgo.ApplicationCommand{
		Name:        c.Name(),
		Description: c.Description(),
	}
}

func (c *DogCommand) Handler() common.CommandHandler {
	return func(s *discordgo.Session, i *discordgo.InteractionCreate) {
		// Fetch the dog image
		resp, err := http.Get("https://some-random-api.com/img/dog")
		if err != nil {
			c.HandleInteractionRespond(s, i, &discordgo.InteractionResponse{
				Type: discordgo.InteractionResponseChannelMessageWithSource,
				Data: &discordgo.InteractionResponseData{
					Content: "Failed to fetch dog image",
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
