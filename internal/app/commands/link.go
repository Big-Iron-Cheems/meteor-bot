package commands

import (
	"encoding/json"
	"fmt"
	"github.com/bwmarrin/discordgo"
	"meteor-bot/internal/app/common"
	"meteor-bot/internal/app/config"
	"net/http"
	"net/url"
	"strings"
)

type LinkCommand struct {
	common.BaseCommand
}

func NewLinkCommand() *LinkCommand {
	return &LinkCommand{
		BaseCommand: *common.NewCommandBuilder().
			SetName("link").
			SetDescription("Links your Discord account to your Meteor account").
			Build(),
	}
}

func (c *LinkCommand) Build() *discordgo.ApplicationCommand {
	return &discordgo.ApplicationCommand{
		Name:        c.Name(),
		Description: c.Description(),
		Options: []*discordgo.ApplicationCommandOption{
			{
				Name:         "token",
				Description:  "The token generated on the Meteor website",
				Type:         discordgo.ApplicationCommandOptionString,
				Required:     true,
				ChannelTypes: []discordgo.ChannelType{discordgo.ChannelTypeDM},
			},
		},
	}
}

func (c *LinkCommand) Handle(s *discordgo.Session, i *discordgo.InteractionCreate) {
	// If this is not a DM, respond with an ephemeral message
	if i.GuildID != "" {
		c.HandleInteractionRespond(s, i, &discordgo.InteractionResponse{
			Type: discordgo.InteractionResponseChannelMessageWithSource,
			Data: &discordgo.InteractionResponseData{
				Content: "This command can only be used in DMs.",
				Flags:   discordgo.MessageFlagsEphemeral,
			},
		})
		return
	}

	// Get the token from the options
	token := i.ApplicationCommandData().Options[0].StringValue()
	if len(token) == 0 {
		c.HandleInteractionRespond(s, i, &discordgo.InteractionResponse{
			Type: discordgo.InteractionResponseChannelMessageWithSource,
			Data: &discordgo.InteractionResponseData{
				Content: "You must provide a valid token.",
				Flags:   discordgo.MessageFlagsEphemeral,
			},
		})
		return
	}

	// API request to link the Discord account
	// TODO: make a util function for the request
	userId := i.User.ID
	req, err := http.NewRequest("POST", fmt.Sprintf("%s/account/linkDiscord", config.GlobalConfig.ApiBase), strings.NewReader(url.Values{
		"id":    {userId},
		"token": {token},
	}.Encode()))
	if err != nil {
		c.HandleInteractionRespond(s, i, &discordgo.InteractionResponse{
			Type: discordgo.InteractionResponseChannelMessageWithSource,
			Data: &discordgo.InteractionResponseData{
				Content: "Failed to create request.",
				Flags:   discordgo.MessageFlagsEphemeral,
			},
		})
		return
	}
	req.Header.Set("Authorization", config.GlobalConfig.BackendToken)

	resp, err := http.DefaultClient.Do(req)
	if err != nil {
		c.HandleInteractionRespond(s, i, &discordgo.InteractionResponse{
			Type: discordgo.InteractionResponseChannelMessageWithSource,
			Data: &discordgo.InteractionResponseData{
				Content: "Failed to link your Discord account. Please try again later.",
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
				Content: "Failed to link your Discord account. Try generating a new token by refreshing the account page and clicking the link button again.",
				Flags:   discordgo.MessageFlagsEphemeral,
			},
		})
		return
	}

	// Respond to the interaction
	c.HandleInteractionRespond(s, i, &discordgo.InteractionResponse{
		Type: discordgo.InteractionResponseChannelMessageWithSource,
		Data: &discordgo.InteractionResponseData{
			Content: "Successfully linked your Discord account.",
			Flags:   discordgo.MessageFlagsEphemeral,
		},
	})
}
