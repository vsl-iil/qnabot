use std::error::Error;

use frankenstein::{AnswerCallbackQueryParams, Api, BotCommand, EditMessageReplyMarkupParams, GetUpdatesParams, GetUpdatesParamsBuilder, InlineKeyboardButton, InlineKeyboardMarkup, KeyboardButton, MaybeInaccessibleMessage, Message, MessageEntity, MethodResponse, ReplyKeyboardMarkup, ReplyMarkup, SendMessageParams, SendMessageParamsBuilder, SetMyCommandsParams, TelegramApi, UpdateContent};

use crate::arenatree::NodeId;
use crate::db;
use crate::util::logging::{check_result, check_pass, non_fatal, fatal};
use crate::serde::QASerde;
use crate::error::telegram::*;

static ROOT: NodeId = 0;

pub struct TelegramSender {
    api: Api,
    database_rw: QASerde,
    send_params: SendMessageParamsBuilder,
    update_params: GetUpdatesParamsBuilder,
    reply_markup: ReplyMarkup,
    question_db: db::Database,
}

impl TelegramSender {
    /// Конструктор, создающий нового бота
    pub fn new(token: &str, dbpath: &str, question_db: db::Database) -> Self {
        let api = Api::new(token);

        Self::build_commands(&api);

        let db = QASerde::new().build(dbpath).unwrap();

        match Self::build_inline_keyboard() {
            Ok(keyboard_markup) =>  {
                TelegramSender {
                    api,
                    database_rw: db,
                    send_params: SendMessageParams::builder(),
                    update_params: GetUpdatesParams::builder(),
                    reply_markup: ReplyMarkup::InlineKeyboardMarkup(keyboard_markup),
                    question_db,
                }
            }
            Err(error) => {
                fatal(error);
                unreachable!()
            }
        }
            
    }

    fn build_choice_keyboard(&mut self, parent: Option<String>) -> Result<(), Box<dyn Error>> {
        eprintln!("Строим клавиатуру с родителем {parent:?}");
        let mut choice_keyboard: Vec<Vec<KeyboardButton>> = vec![];

        let children = self.database_rw.get_children(parent)?;
        for question in children {
            choice_keyboard.push(vec![KeyboardButton::builder().text(question).build()]);
        }
        eprintln!("Элементы клавиатуры: {choice_keyboard:?}");

        self.reply_markup = ReplyMarkup::ReplyKeyboardMarkup(ReplyKeyboardMarkup::builder().keyboard(choice_keyboard).build());
        Ok(())
    }

    fn build_inline_keyboard() -> Result<InlineKeyboardMarkup, <frankenstein::Api as TelegramApi>::Error> {
        let mut inline_keyboard: Vec<Vec<InlineKeyboardButton>> = vec![];
        inline_keyboard.push(vec![
            InlineKeyboardButton::builder().text("Да").callback_data("save").build(),
            InlineKeyboardButton::builder().text("Нет").callback_data("nosave").build(),
        ]);

        Ok(InlineKeyboardMarkup::builder().inline_keyboard(inline_keyboard).build())
    }

    fn build_commands(api: &Api) -> Result<(), <frankenstein::Api as TelegramApi>::Error> {
        let mut commandvec: Vec<BotCommand> = vec![];
        commandvec.push(BotCommand::builder()
                        .command("start")
                        .description("Начать работу")
        .build());
        commandvec.push(BotCommand::builder()
                        .command("reset")
                        .description("Вернуться в начало")
        .build());
        commandvec.push(BotCommand::builder()
                        .command("help")
                        .description("Помощь")
        .build());

        let commandparams = SetMyCommandsParams::builder().commands(commandvec).build();

        let _ = api.set_my_commands(&commandparams)?;

        Ok(())
    }

    pub fn start_reply_loop(&mut self) {
        let mut built_update_params: GetUpdatesParams;
        built_update_params = self.update_params.clone().build();

        let mut prev_message: Option<Message> = None;

        loop {
            let result = self.api
                             .get_updates(&built_update_params);

            match result {
                Ok(response) => {
                    for update in response.result {
                        match update.content {
                            // Сообщение
                            UpdateContent::Message(message) 
                                => {
                                    prev_message = Some(message.clone());
                                    check_result(self.process_message(message), non_fatal);
                                },

                            // Ввод пользователя посредством кнопок
                            UpdateContent::CallbackQuery(callback) 
                                => {
                                // Сохраняем вопрос, если пользователь хочет        
                                if let Some(save_option) = callback.data {
                                    if save_option == "save" {
                                        if let Some(message) = prev_message.take() {
                                            self.save_question(message);
                                        }
                                    }
                                }
                                // Ответ пользователю, что мы всё обработали
                                let callback_params = AnswerCallbackQueryParams::builder()
                                                      .text("Я сохранил твой выбор, спасибо!")
                                                      .callback_query_id(callback.id)
                                                      .build();
                                check_result(self.api.answer_callback_query(&callback_params), non_fatal);

                                // Убираем кнопки с предыдущего сообщения
                                if let Some(msg) = callback.message {
                                    use MaybeInaccessibleMessage::{Message, InaccessibleMessage};
                                    let (chat_id, msg_id) = match msg 
                                    {
                                        Message(message) => (message.chat.id, message.message_id),
                                        InaccessibleMessage(message) => (message.chat.id, message.message_id),
                                    };
                                    let edit_message_params = EditMessageReplyMarkupParams::builder()
                                                              .chat_id(chat_id)
                                                              .message_id(msg_id)
                                                              .reply_markup(InlineKeyboardMarkup::builder()
                                                                            .inline_keyboard(vec![vec![]])
                                                                            .build())
                                                              .build();

                                    check_result(self.api.edit_message_reply_markup(&edit_message_params), non_fatal);
                                }
                            }

                            _ => (),
                        }

                        built_update_params = self.update_params.clone().offset(update.update_id + 1).build();
                    }
                }
                Err(error) => {
                    non_fatal(error);
                }
            }
        }
    }

    fn process_message(&mut self, message: Message) -> Result<(), frankenstein::Error> {
        // Наверное, вопрос, ищем ответ
        if let Some(question) = message.text {
            // Проверяем, если пользователь отправил команду, и отправляем 
            // соответствующий ответ, если да
            if let Some(entities) = message.entities {
                if let Some(cmd_answer) = self.get_command_reply(entities, &question) {
                    self.send_message(message.chat.id, cmd_answer)?;
                    return Ok(());
                }
            }
            // Не команда. Значит, вопрос?
            if let Some(answer) = self.query_question(question.clone()) {
                self.send_message(message.chat.id, answer)?;
            } else if self.database_rw.contains(&question) {
                // Ответ не найден, но есть такая категория
                self.send_message(message.chat.id, format!("Категория: \"{}\"",question))?;
            } else {
                // Пользователь ввёл что-то невнятное. На всякий случай запишем
                // Создадим клавиатуру для выбора юзера
                match Self::build_inline_keyboard() {
                    Ok(keyboard) => self.reply_markup = ReplyMarkup::InlineKeyboardMarkup(keyboard),
                    Err(error) => non_fatal(error),
                }
                self.send_message(
                    message.chat.id, 
                    "Прости, я не знаю ответа на твой вопрос...".to_string())?;
            }
        } else {
            // Пользователь зачем-то отправил что-то 
            // другое вместо текстового сообщения
            self.send_message(
                message.chat.id, 
                "Извини, я понимаю только вопросы текстом!".to_string()
            )?;
        }

        return Ok(());
    }

    fn get_command_reply(&mut self, entities: Vec<MessageEntity>, message: &str) -> Option<String> {
        for e in entities {
            match e.type_field {
                frankenstein::MessageEntityType::BotCommand => {
                    return Some(self.match_command(message))
                }
                _ => continue,
            }
        }

        None
    }

    fn reset_choice_keyboard(&mut self) -> Result<(),Box<dyn Error>>  {
        eprintln!("Клавиатура сброшена");
        self.build_choice_keyboard(None)?;

        Ok(())
    }

    fn match_command(&mut self, msg: &str) -> String {
        match msg {
            "/start" => {
                self.reset_choice_keyboard();
                "Привет! Я бот!"
            },
            "/help"  => "Выбери вопрос или напиши свой",
            "/reset" => {
                self.reset_choice_keyboard();
                "Вернулись в начало."
            },
            _        => "Неизвестная команда",
        }
        .to_string()
    }

    fn send_message(&self, id: i64, message: String) 
    -> Result<MethodResponse<Message>, frankenstein::Error> 
    {
        let send_params_builder = SendMessageParams::builder()
                          .chat_id(id)
                          .text(message);

        let send_params: SendMessageParams;
        // if let Some(markup) = reply_markup {
        send_params = send_params_builder.reply_markup(self.reply_markup.clone()).build();
        // } else {
            // send_params = send_params_builder.build();
        // }
            
        eprintln!("Отправляем с reply_markup {:?}", self.reply_markup);
        self.api.send_message(&send_params)
    }

    fn query_question(&mut self, question: String) -> Option<String> {
        let children = check_pass(self.database_rw.get_children(Some(question.clone())), non_fatal);
        // Найдены дети текущего вопроса
        if let Some(arr) = children {
            // Ребёнок один и он является ответом
            if arr.len() == 1 && self.database_rw.is_question(&question) {
                self.reset_choice_keyboard();
                return arr.get(0).cloned();
            } 
            // Дети являются категориями, идём глубже
            else {
                self.build_choice_keyboard(Some(question));
            }
        } 
        else {
            // Детей нет - не знаем такого вопроса/категории
            self.reset_choice_keyboard();
        }

        None
    }

    fn save_question(&mut self, message: Message) {
            // Сохраняем вопрос 
            eprintln!("Cохраняем вопрос: {message:?}");
            if let Some(message_text) = &message.text {
                let user_id = message.from.as_ref().map_or(0, |x| x.id);
                check_result(self.question_db.insert_question(&message_text, user_id), non_fatal);
            }
    }
}


