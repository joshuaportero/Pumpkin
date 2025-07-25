use async_trait::async_trait;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_protocol::java::client::play::CTransfer;
use pumpkin_util::text::TextComponent;
use pumpkin_util::text::color::{Color, NamedColor};

use crate::command::args::bounded_num::BoundedNumArgumentConsumer;
use crate::command::args::players::PlayersArgumentConsumer;
use crate::command::args::simple::SimpleArgConsumer;
use crate::command::args::{Arg, FindArgDefaultName};
use crate::command::dispatcher::CommandError::{InvalidConsumption, InvalidRequirement};
use crate::command::tree::builder::{argument, argument_default_name, require};
use crate::command::{
    CommandError, CommandExecutor, CommandSender, args::ConsumedArgs, tree::CommandTree,
};

const NAMES: [&str; 1] = ["transfer"];

const DESCRIPTION: &str = "Triggers a transfer of a player to another server.";

const ARG_HOSTNAME: &str = "hostname";

const ARG_PLAYERS: &str = "players";

fn port_consumer() -> BoundedNumArgumentConsumer<i32> {
    BoundedNumArgumentConsumer::new()
        .name("port")
        .min(1)
        .max(65535)
}

struct TargetSelfExecutor;

#[async_trait]
impl CommandExecutor for TargetSelfExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender,
        _server: &crate::server::Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        let Some(Arg::Simple(hostname)) = args.get(ARG_HOSTNAME) else {
            return Err(InvalidConsumption(Some(ARG_HOSTNAME.into())));
        };

        let port = match port_consumer().find_arg_default_name(args) {
            Err(_) => 25565,
            Ok(Ok(count)) => count,
            Ok(Err(_)) => {
                sender
                    .send_message(
                        TextComponent::text("Port must be between 1 and 65535.")
                            .color(Color::Named(NamedColor::Red)),
                    )
                    .await;
                return Ok(());
            }
        };

        if let CommandSender::Player(player) = sender {
            let name = &player.gameprofile.name;
            log::info!("[{name}: Transferring {name} to {hostname}:{port}]");
            player
                .client
                .enqueue_packet(&CTransfer::new(hostname, VarInt(port)))
                .await;
            Ok(())
        } else {
            Err(InvalidRequirement)
        }
    }
}

struct TargetPlayerExecutor;

#[async_trait]
impl CommandExecutor for TargetPlayerExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender,
        _server: &crate::server::Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        let Some(Arg::Simple(hostname)) = args.get(ARG_HOSTNAME) else {
            return Err(InvalidConsumption(Some(ARG_HOSTNAME.into())));
        };

        let port = match port_consumer().find_arg_default_name(args) {
            Err(_) => 25565,
            Ok(Ok(count)) => count,
            Ok(Err(_)) => {
                sender
                    .send_message(
                        TextComponent::text("Port must be between 1 and 65535.")
                            .color(Color::Named(NamedColor::Red)),
                    )
                    .await;
                return Ok(());
            }
        };

        let Some(Arg::Players(players)) = args.get(ARG_PLAYERS) else {
            return Err(InvalidConsumption(Some(ARG_PLAYERS.into())));
        };

        for p in players {
            p.client
                .enqueue_packet(&CTransfer::new(hostname, VarInt(port)))
                .await;
            log::info!(
                "[{sender}: Transferring {} to {hostname}:{port}]",
                p.gameprofile.name
            );
        }

        Ok(())
    }
}

#[allow(clippy::redundant_closure_for_method_calls)]
pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION).then(
        argument(ARG_HOSTNAME, SimpleArgConsumer)
            .then(require(|sender| sender.is_player()).execute(TargetSelfExecutor))
            .then(
                argument_default_name(port_consumer())
                    .then(require(|sender| sender.is_player()).execute(TargetSelfExecutor))
                    .then(
                        argument(ARG_PLAYERS, PlayersArgumentConsumer)
                            .execute(TargetPlayerExecutor),
                    ),
            ),
    )
}
