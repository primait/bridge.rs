FROM prima/rust:1.44.1-1

WORKDIR /code

COPY entrypoint /code/entrypoint

RUN chown -R app:app /code

# Serve per avere l'owner dei file scritti dal container uguale all'utente Linux sull'host
USER app

ENTRYPOINT ["./entrypoint"]
