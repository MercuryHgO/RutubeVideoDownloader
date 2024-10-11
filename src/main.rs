use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, USER_AGENT};
use serde_json::Value;
use teloxide::{requests::Requester, types::{InputFile, Message}, utils::command::BotCommands, Bot};
use std::{fs::{self, File}, io::Write, path::Path};

#[derive(BotCommands)]
#[command(rename_rule = "lowercase", description = "Доступные команды:")]
enum Command {
    #[command(description = "Начать")]
    Start,
    #[command(description = "Помощь")]
    Help,
    #[command(description = "Скачать видео")]
    Download(String),
}

///Принимает URL видео в качестве параметра и возвращает кортеж, содержащий следующие данные:
///
///    Автор видео (строка)
///    Название видео (строка)
///    Ссылка на потоковое видео в формате m3u8 (строка)
///
///Основные шаги выполнения функции:
///
///    Создание HTTP-клиента: Используется библиотека reqwest для создания клиента, который будет отправлять запросы.
///    Настройка заголовков: Устанавливаются заголовки HTTP, включая User-Agent и Accept, чтобы имитировать запрос от браузера.
///    Формирование API URL: Извлекается часть URL, чтобы сформировать запрос к API Rutube для получения информации о видео.
///    Отправка запроса: Выполняется GET-запрос к сформированному API URL, и полученный ответ парсится как JSON.
///    Извлечение данных:
///        Извлекается имя автора видео.
///        Извлекается название видео.
///        Извлекается ссылка на потоковое видео в формате m3u8.
///    Очистка данных: Удаляются специальные символы из имени автора и названия видео, а пробелы в названии заменяются на подчеркивания.
///    Вывод m3u8: Ссылка на потоковое видео выводится в консоль.
///    Возврат данных: Функция возвращает кортеж с именем автора, названием видео и ссылкой на m3u8.
async fn get_m3u8_list(url: &str) -> (String, String, String) {
    let client = reqwest::Client::new();
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/98.0.4758.132 YaBrowser/22.3.1.892 Yowser/2.5 Safari/537.36"));
    headers.insert(ACCEPT, HeaderValue::from_static("*/*"));

    let url = url
        .split('/')
        .map(|split| split)
        .collect::<Vec<&str>>();

    let api_url = format!(
        "https://rutube.ru/api/play/options/{}/?no_404=true&referer=https%3A%2F%2Frutube.ru",
        url[url.len() - 2]
    );

    let response: Value = client
        .get(api_url)
        .headers(headers)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    

    let video_author = response["author"]["name"].as_str()
        .unwrap_or("")
        .replace(&['/', '\\', '[', ']', '?', '\'', '"', ':', '.'][..], "");
    let video_title = response["title"].as_str()
        .unwrap_or("")
        .replace(&['/', '\\', '[', ']', '?', '\'', '"', ':', '.'][..], "")
        .replace(" ", "_");
    let video_m3u8 = response["video_balancer"]["m3u8"].as_str().unwrap_or("").to_string();

    println!("{}",video_m3u8);
    (video_author, video_title, video_m3u8)
}

///Принимает URL в формате m3u8 в качестве параметра и возвращает строку, которая представляет собой ссылку на один из сегментов медиафайла.
///Основные шаги выполнения функции:
///
///    Проверка существования директории:
///        Функция проверяет, существует ли директория с именем seg. Если она не существует, создается новая директория с помощью fs::create_dir("seg").
///    Отправка GET-запроса:
///        Используется библиотека reqwest для отправки асинхронного GET-запроса по указанному URL m3u8. Ответ ожидается в текстовом формате.
///    Запись ответа в файл:
///        Полученный текст (содержимое файла m3u8) записывается в файл pl_list.txt, который создается в директории seg. Для записи используется функция writeln!.
///    Чтение содержимого файла:
///        Содержимое файла pl_list.txt считывается обратно в программу с помощью fs::read_to_string.
///    Обработка строк:
///        Содержимое файла разбивается на строки, которые собираются в вектор lines. Это позволяет работать с каждой строкой отдельно.
///    Возврат ссылки:
///        Функция возвращает предпоследнюю строку из вектора lines, что обычно соответствует одной из ссылок на сегменты медиафайла.
async fn get_link_from_m3u8(url_m3u8: &str) -> String {
    if !Path::new("seg").exists() {
        fs::create_dir("seg").unwrap();
    }
    
    let response = reqwest::get(url_m3u8)
        .await.unwrap()
        .text()
        .await.unwrap();
    let mut file = File::create("seg/pl_list.txt").unwrap();
    writeln!(file, "{}", response).unwrap();

    let src = fs::read_to_string("seg/pl_list.txt").unwrap();
    let lines: Vec<&str> = src.lines().collect();
    println!("{lines:?}");

    lines[lines.len() - 2].to_string()
}

///Принимает URL плейлиста в формате m3u8 и возвращает количество сегментов как целое число (usize).
///Основные шаги выполнения функции:
///
///    Отправка GET-запроса:
///        Используется библиотека reqwest для отправки асинхронного GET-запроса по указанному URL m3u8. Ответ ожидается в текстовом формате.
///    Обработка ответа:
///        Полученный текст (содержимое файла m3u8) разбивается на строки, которые собираются в вектор lines.
///    Подсчет сегментов:
///        Функция возвращает количество строк в векторе lines, вычитая 2. Это делается с предположением, что последние две строки не содержат информации о сегментах (обычно это может быть комментарий или конец плейлиста).
async fn get_segment_count(m3u8_link: &str) -> usize {
    let response = reqwest::get(m3u8_link)
        .await.unwrap()
        .text()
        .await.unwrap();

    let lines: Vec<&str> = response.lines().collect();
    
    lines.len() - 2 // Assuming last line is the playlist end
}


///Предназначена для формирования базового URL для загрузки сегментов видео из плейлиста m3u8. Она принимает ссылку на файл m3u8 и возвращает строку, представляющую базовый URL для сегментов.
///Основные шаги выполнения функции:
///
///    Разделение ссылки:
///        Функция принимает строку m3u8_link и использует метод split для разделения строки по подстроке .m3u8. Это позволяет получить часть URL, которая предшествует расширению .m3u8.
///    Получение первой части:
///        Метод next() возвращает первый элемент итератора, созданного методом split. Поскольку мы уверены, что .m3u8 присутствует в строке, мы используем unwrap() для получения значения. Если по каким-то причинам .m3u8 отсутствует, это приведет к панике.
///    Формирование нового URL:
///        Функция добавляет / к полученной части URL, создавая базовый путь для доступа к сегментам видео.
fn get_download_link(m3u8_link: &str) -> String {
    format!("{}/", m3u8_link.split(".m3u8").next().unwrap())
}

///Предназначена для загрузки сегментов видео из потока, используя ссылки на сегменты, сформированные по заданному URL. Она также взаимодействует с Telegram-ботом для информирования пользователя о процессе загрузки.
///Основные шаги выполнения функции:
///
///    Отправка сообщения о начале загрузки:
///        Функция отправляет сообщение пользователю через бота, информируя о начале процесса скачивания видео.
///    Проверка и создание директории:
///        Проверяется существование директории seg. Если она не существует, создается новая директория для хранения загружаемых сегментов.
///    Цикл загрузки сегментов:
///        Функция проходит в цикле по количеству сегментов (count), начиная с 1 до count включительно.
///        Для каждого сегмента:
///            Обновляется сообщение о текущем статусе загрузки.
///            Формируется URL для сегмента, используя переданный link и номер сегмента.
///            Инициализируется переменная tries, которая отслеживает количество попыток загрузки сегмента (максимум 3).
///    Попытка загрузки сегмента:
///        Выполняется асинхронный GET-запрос для загрузки сегмента.
///        Если запрос успешен, цикл завершает свою работу.
///        Если происходит ошибка, количество оставшихся попыток уменьшается, и бот обновляет сообщение с информацией об ошибке и оставшихся попытках.
///    Запись сегмента в файл:
///        После успешной загрузки содержимое сегмента записывается в файл с именем segment-<номер>-v1-a1.ts в директории seg.
///    Завершение процесса:
///        После завершения загрузки всех сегментов выводится сообщение в консоль о том, что все сегменты были успешно загружены.
async fn get_download_segment(link: &str, count: usize, bot: &Bot,message: &Message) {
    let status: Message = bot.send_message(message.chat.id, "Скачивание видео...").await.unwrap();

    if !Path::new("seg").exists() {
        fs::create_dir("seg").unwrap();
    }

    for item in 1..=count {
        let mut tries = 3;
        bot
            .edit_message_text(message.chat.id, status.id, format!("Загружаю сегмент {}/{}", item, count))
            .await.unwrap();
        let segment_url = format!("{}segment-{}-v1-a1.ts", link, item);
        let mut response = reqwest::get(&segment_url)
            .await.unwrap()
            .bytes()
            .await;
        while tries > 0 {
            match response {
                Ok(_) => {
                    break
                },
                Err(_) => {
                    tries -= 1;
                    bot
                        .edit_message_text(message.chat.id, status.id, 
                            format!("Ошибка загрузки сегмента {}/{}, осталось попыток: {}", item, count,tries))
                        .await.unwrap();
                    response = reqwest::get(&segment_url)
                        .await.unwrap()
                        .bytes()
                        .await;
                },
            }
        }
        let mut file = File::create(format!("seg/segment-{}-v1-a1.ts", item)).unwrap();
        file.write_all(&response.unwrap()).unwrap();
    }
    
    println!("[INFO] - Все сегменты загружены");
}

///Она принимает имя автора, название видео и количество сегментов, а затем выполняет следующие шаги:
///Основные шаги выполнения функции:
///
///    Проверка и создание директории:
///        Функция проверяет, существует ли директория с именем автора (author). Если она не существует, создается новая директория для хранения конечного видео.
///    Создание выходного файла:
///        Формируется путь к выходному файлу, который будет содержать объединенные сегменты, с использованием названия видео (title). Выходной файл создается в формате .ts.
///    Объединение сегментов:
///        В цикле от 1 до count функция:
///            Формирует путь к каждому сегменту (segment-<номер>-v1-a1.ts).
///            Читает содержимое сегмента с помощью fs::read.
///            Записывает содержимое каждого сегмента в выходной файл с помощью write_all.
///    Конвертация в MP4:
///        После объединения всех сегментов вызывается команда ffmpeg для конвертации выходного файла .ts в файл формата .mp4.
///        Используется метод std::process::Command для выполнения команды, передавая необходимые аргументы.
///    Удаление временной директории:
///        После завершения конвертации временная директория seg удаляется с помощью fs::remove_dir_all.
///    Возврат пути к конечному файлу:
///        Функция возвращает строку, представляющую путь к созданному файлу MP4.
fn merge_ts(author: &str, title: &str, count: usize) -> String {
    if !Path::new(author).exists() {
        fs::create_dir(author).unwrap();
    }

    let output_file_path = format!("seg/{}.ts", title);
    let mut merged_file = File::create(&output_file_path).unwrap();

    for ts in 1..=count {
        let segment_path = format!("seg/segment-{}-v1-a1.ts", ts);
        let content = fs::read(segment_path).unwrap();
        merged_file.write_all(&content).unwrap();
    }

    std::process::Command::new("ffmpeg")
        .args(&["-i", &output_file_path, "-c", "copy", &format!("{}/{}.mp4", author, title)])
        .output()
        .expect("[+] - Конвертирование завершено");

    fs::remove_dir_all("seg").unwrap();

    format!("{}/{}.mp4", author, title)
}

#[tokio::main]
async fn main() {
    let bot = Bot::from_env(); // Инициализая бота с помощью переменной среды окруженеия

    teloxide::repl(bot.clone(), |bot: Bot, message: Message| async move {
        match Command::parse(message.text().unwrap(), "bot_name") {
            Ok(command) => {
                match command {
                    Command::Start => {
                        bot.send_message(message.chat.id, "Чтобы скачать видео, используйте комманду /download [ссылка]").await?;
                    },
                    Command::Help => {
                        bot.send_message(message.chat.id, "Используйте комманду /download [ссылка] для того чтобы скачать видео").await?;
                    },
                    Command::Download(url) => {
                        let m3u8_url = get_m3u8_list(&url).await;
                        let m3u8_link = get_link_from_m3u8(&m3u8_url.2).await;
                        let seg_count = get_segment_count(&m3u8_link).await;
                        let dwnl_link = get_download_link(&m3u8_link);
                        get_download_segment(&dwnl_link, seg_count, &bot, &message).await;
                        let path = merge_ts(&m3u8_url.0, &m3u8_url.1, seg_count);
                        
                        let file = InputFile::file(path);

                        bot.send_video(message.chat.id, file).await?;
                    },
                }
            },
            _ => {
                bot.send_message(message.chat.id, "Не понял комманду".to_string()).await?;
            },
        }
        Ok(())        
    }).await;
}
