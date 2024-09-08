package silly

import (
	"encoding/json"
	"github.com/bwmarrin/discordgo"
	"math/rand"
	"meteor-bot/internal/app/common"
	"net/http"
)

type PandaCommand struct {
	common.BaseCommand
}

func NewPandaCommand() *PandaCommand {
	return &PandaCommand{
		BaseCommand: *common.NewCommandBuilder().
			SetName("panda").
			SetDescription("funny thing").
			Build(),
	}
}

func (c *PandaCommand) Build() *discordgo.ApplicationCommand {
	return &discordgo.ApplicationCommand{
		Name:        c.Name(),
		Description: c.Description(),
	}
}

func (c *PandaCommand) Handle(s *discordgo.Session, i *discordgo.InteractionCreate) {
	animal := "panda"
	if rand.Intn(2) == 0 {
		animal = "red_panda"
	}

	// Fetch the panda image
	resp, err := http.Get("https://some-random-api.com/img/" + animal)
	if err != nil {
		c.HandleInteractionRespond(s, i, &discordgo.InteractionResponse{
			Type: discordgo.InteractionResponseChannelMessageWithSource,
			Data: &discordgo.InteractionResponseData{
				Content: "Failed to fetch panda image",
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
