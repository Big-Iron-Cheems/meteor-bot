package silly

import (
	"fmt"
	"github.com/bwmarrin/discordgo"
	"math/rand"
	"meteor-bot/internal/app/common"
)

type MonkeyCommand struct {
	common.BaseCommand
}

func NewMonkeyCommand() *MonkeyCommand {
	return &MonkeyCommand{
		BaseCommand: *common.NewCommandBuilder().
			SetName("monkey").
			SetDescription("monke").
			Build(),
	}
}

func (c *MonkeyCommand) Build() *discordgo.ApplicationCommand {
	return &discordgo.ApplicationCommand{
		Name:        c.Name(),
		Description: c.Description(),
	}
}

func (c *MonkeyCommand) Handler() common.CommandHandler {
	return func(s *discordgo.Session, i *discordgo.InteractionCreate) {
		w := rand.Intn(801) + 200
		h := rand.Intn(801) + 200

		url := fmt.Sprintf("https://www.placemonkeys.com/%d/%d?random", w, h)

		c.HandleInteractionRespond(s, i, &discordgo.InteractionResponse{
			Type: discordgo.InteractionResponseChannelMessageWithSource,
			Data: &discordgo.InteractionResponseData{
				Content: url,
			},
		})
	}
}
